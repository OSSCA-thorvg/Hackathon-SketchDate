use std::{
    collections::{HashMap, HashSet},
    env,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use axum::{
    Router,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::{HeaderMap, StatusCode, header::ORIGIN},
    response::{IntoResponse, Response},
    routing::get,
};
use futures_util::{SinkExt, StreamExt};
use rand::Rng;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::{Mutex, mpsc};
use uuid::Uuid;

const PROTOCOL_VERSION: u8 = 1;
const DEFAULT_PORT: u16 = 3000;
const DEFAULT_ROUND_SECONDS: u64 = 180;
const DEFAULT_DECISION_SECONDS: u64 = 20;
const DEFAULT_TURN_TTL_SECONDS: u64 = 3_600;

type ConnectionId = u64;
type Outbound = mpsc::UnboundedSender<ServerMessage>;

#[derive(Clone)]
struct AppState {
    inner: Arc<Mutex<ServerState>>,
    next_connection_id: Arc<AtomicU64>,
    turn: TurnService,
    allowed_origins: Arc<HashSet<String>>,
    round_duration: Duration,
    decision_duration: Duration,
}

impl AppState {
    fn from_env() -> Result<Self> {
        let allowed_origins = env::var("ALLOWED_ORIGINS")
            .unwrap_or_else(|_| {
                "http://localhost:5173,http://127.0.0.1:5173,http://localhost:4173".to_owned()
            })
            .split(',')
            .map(str::trim)
            .filter(|origin| !origin.is_empty())
            .map(ToOwned::to_owned)
            .collect();

        Ok(Self {
            inner: Arc::new(Mutex::new(ServerState::default())),
            next_connection_id: Arc::new(AtomicU64::new(1)),
            turn: TurnService::from_env()?,
            allowed_origins: Arc::new(allowed_origins),
            round_duration: duration_from_env("AVATAR_ROUND_SECONDS", DEFAULT_ROUND_SECONDS)?,
            decision_duration: duration_from_env("DECISION_SECONDS", DEFAULT_DECISION_SECONDS)?,
        })
    }

    fn origin_allowed(&self, headers: &HeaderMap) -> bool {
        let Some(origin) = headers.get(ORIGIN) else {
            // Non-browser clients do not necessarily send Origin. Browser WebSockets do.
            return true;
        };

        origin
            .to_str()
            .is_ok_and(|origin| self.allowed_origins.contains(origin))
    }
}

#[derive(Default)]
struct ServerState {
    connections: HashMap<ConnectionId, Connection>,
    waiting: Vec<ConnectionId>,
    rooms: HashMap<String, Room>,
}

struct Connection {
    outbound: Outbound,
    queued: bool,
    room_id: Option<String>,
}

struct Room {
    participants: [ConnectionId; 2],
    decisions: HashMap<ConnectionId, Decision>,
    phase: RoomPhase,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RoomPhase {
    AvatarDate,
    Decision,
    Revealed,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum Decision {
    Reveal,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DecisionResult {
    Pending,
    Reveal,
    End,
}

#[derive(Deserialize)]
struct ClientMessage {
    v: u8,
    #[serde(rename = "type")]
    message_type: String,
    #[serde(rename = "roomId")]
    room_id: Option<String>,
    payload: Option<Value>,
}

#[derive(Clone, Serialize)]
struct ServerMessage {
    v: u8,
    #[serde(rename = "type")]
    message_type: &'static str,
    #[serde(rename = "roomId", skip_serializing_if = "Option::is_none")]
    room_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload: Option<Value>,
}

impl ServerMessage {
    fn new(message_type: &'static str, room_id: Option<&str>, payload: Option<Value>) -> Self {
        Self {
            v: PROTOCOL_VERSION,
            message_type,
            room_id: room_id.map(ToOwned::to_owned),
            payload,
        }
    }

    fn error(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(
            "server.error",
            None,
            Some(json!({ "code": code, "message": message.into() })),
        )
    }
}

struct MatchCreated {
    room_id: String,
    participants: [(ConnectionId, Outbound); 2],
}

#[derive(Clone)]
struct TurnService {
    client: Client,
    key_id: Option<String>,
    api_token: Option<String>,
    ttl_seconds: u64,
}

impl TurnService {
    fn from_env() -> Result<Self> {
        Ok(Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .context("failed to build the Cloudflare TURN HTTP client")?,
            key_id: env::var("TURN_KEY_ID").ok(),
            api_token: env::var("TURN_KEY_API_TOKEN").ok(),
            ttl_seconds: env::var("TURN_TTL_SECONDS")
                .ok()
                .map(|value| value.parse())
                .transpose()
                .context("TURN_TTL_SECONDS must be an integer")?
                .unwrap_or(DEFAULT_TURN_TTL_SECONDS),
        })
    }

    async fn credentials(&self) -> Result<Value> {
        let key_id = self
            .key_id
            .as_deref()
            .context("TURN_KEY_ID is not configured")?;
        let api_token = self
            .api_token
            .as_deref()
            .context("TURN_KEY_API_TOKEN is not configured")?;
        let url = format!(
            "https://rtc.live.cloudflare.com/v1/turn/keys/{key_id}/credentials/generate-ice-servers"
        );

        let response = self
            .client
            .post(url)
            .bearer_auth(api_token)
            .json(&json!({ "ttl": self.ttl_seconds }))
            .send()
            .await
            .context("Cloudflare TURN credential request failed")?
            .error_for_status()
            .context("Cloudflare rejected the TURN credential request")?;

        response
            .json()
            .await
            .context("Cloudflare returned invalid TURN credential JSON")
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let state = AppState::from_env()?;
    let port = env::var("PORT")
        .ok()
        .map(|value| value.parse())
        .transpose()
        .context("PORT must be a valid TCP port")?
        .unwrap_or(DEFAULT_PORT);
    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/ws", get(websocket_handler))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;

    println!("SketchDate backend listening on {port}");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn healthz() -> &'static str {
    "ok"
}

async fn websocket_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    websocket: WebSocketUpgrade,
) -> Response {
    if !state.origin_allowed(&headers) {
        return (StatusCode::FORBIDDEN, "origin is not allowed").into_response();
    }

    websocket
        .on_upgrade(move |socket| handle_socket(state, socket))
        .into_response()
}

async fn handle_socket(state: AppState, socket: WebSocket) {
    let connection_id = state.next_connection_id.fetch_add(1, Ordering::Relaxed);
    let (outbound, mut outbound_rx) = mpsc::unbounded_channel();

    state.inner.lock().await.connections.insert(
        connection_id,
        Connection {
            outbound,
            queued: false,
            room_id: None,
        },
    );

    let (mut socket_tx, mut socket_rx) = socket.split();
    let writer = tokio::spawn(async move {
        while let Some(message) = outbound_rx.recv().await {
            let Ok(json) = serde_json::to_string(&message) else {
                continue;
            };
            if socket_tx.send(Message::Text(json.into())).await.is_err() {
                break;
            }
        }
    });

    while let Some(Ok(message)) = socket_rx.next().await {
        match message {
            Message::Text(text) => process_text(&state, connection_id, &text).await,
            Message::Close(_) => break,
            Message::Ping(_) | Message::Pong(_) | Message::Binary(_) => {}
        }
    }

    disconnect(&state, connection_id).await;
    writer.abort();
}

async fn process_text(state: &AppState, connection_id: ConnectionId, text: &str) {
    let message: ClientMessage = match serde_json::from_str(text) {
        Ok(message) => message,
        Err(_) => {
            send_to_connection(
                state,
                connection_id,
                ServerMessage::error("invalid_message", "message must be valid JSON"),
            )
            .await;
            return;
        }
    };

    if message.v != PROTOCOL_VERSION {
        send_to_connection(
            state,
            connection_id,
            ServerMessage::error(
                "unsupported_version",
                "only protocol version 1 is supported",
            ),
        )
        .await;
        return;
    }

    match message.message_type.as_str() {
        "match.join" => join_matchmaking(state, connection_id).await,
        "match.cancel" => cancel_matchmaking(state, connection_id).await,
        "signal.offer" => relay_signal(state, connection_id, message, "signal.offer").await,
        "signal.answer" => relay_signal(state, connection_id, message, "signal.answer").await,
        "signal.ice" => relay_signal(state, connection_id, message, "signal.ice").await,
        "decision.submit" => submit_decision(state, connection_id, message).await,
        "session.leave" => leave_session(state, connection_id, "left").await,
        _ => {
            send_to_connection(
                state,
                connection_id,
                ServerMessage::error("unknown_type", "unknown message type"),
            )
            .await;
        }
    }
}

async fn join_matchmaking(state: &AppState, connection_id: ConnectionId) {
    let match_created = {
        let mut server = state.inner.lock().await;
        server.join(connection_id)
    };

    let Some(match_created) = match_created else {
        return;
    };

    start_match(state, match_created).await;
}

impl ServerState {
    fn join(&mut self, connection_id: ConnectionId) -> Option<MatchCreated> {
        let connection = self.connections.get_mut(&connection_id)?;
        if connection.queued || connection.room_id.is_some() {
            return None;
        }

        connection.queued = true;
        let _ = connection
            .outbound
            .send(ServerMessage::new("match.queued", None, None));

        self.waiting.retain(|id| self.connections.contains_key(id));
        let opponent_index =
            (!self.waiting.is_empty()).then(|| rand::rng().random_range(0..self.waiting.len()));
        let Some(opponent_index) = opponent_index else {
            self.waiting.push(connection_id);
            return None;
        };
        let opponent_id = self.waiting.swap_remove(opponent_index);
        if opponent_id == connection_id {
            return None;
        }

        let room_id = Uuid::new_v4().to_string();
        let first = self.connections.get_mut(&opponent_id)?;
        first.queued = false;
        first.room_id = Some(room_id.clone());
        let first_outbound = first.outbound.clone();
        let second = self.connections.get_mut(&connection_id)?;
        second.queued = false;
        second.room_id = Some(room_id.clone());
        let second_outbound = second.outbound.clone();

        self.rooms.insert(
            room_id.clone(),
            Room {
                participants: [opponent_id, connection_id],
                decisions: HashMap::new(),
                phase: RoomPhase::AvatarDate,
            },
        );

        Some(MatchCreated {
            room_id,
            participants: [
                (opponent_id, first_outbound),
                (connection_id, second_outbound),
            ],
        })
    }
}

async fn start_match(state: &AppState, created: MatchCreated) {
    let started_at = unix_time_millis();
    let round_seconds = state.round_duration.as_secs();
    for (index, (_, outbound)) in created.participants.iter().enumerate() {
        let _ = outbound.send(ServerMessage::new(
            "match.found",
            Some(&created.room_id),
            Some(json!({ "initiator": index == 0 })),
        ));
        let _ = outbound.send(ServerMessage::new(
            "round.started",
            Some(&created.room_id),
            Some(json!({
                "durationSeconds": round_seconds,
                "startedAt": started_at,
                "endsAt": started_at + round_seconds * 1_000
            })),
        ));
    }

    for (connection_id, _) in created.participants {
        let turn_state = state.clone();
        let turn_room_id = created.room_id.clone();
        tokio::spawn(async move {
            let message = match turn_state.turn.credentials().await {
                Ok(credentials) => {
                    ServerMessage::new("turn.credentials", Some(&turn_room_id), Some(credentials))
                }
                Err(error) => ServerMessage::error("turn_unavailable", error.to_string()),
            };
            send_to_connection(&turn_state, connection_id, message).await;
        });
    }

    let timer_state = state.clone();
    let timer_room_id = created.room_id;
    tokio::spawn(async move {
        tokio::time::sleep(timer_state.round_duration).await;
        open_decision(&timer_state, &timer_room_id).await;
    });
}

async fn cancel_matchmaking(state: &AppState, connection_id: ConnectionId) {
    let mut server = state.inner.lock().await;
    if let Some(connection) = server.connections.get_mut(&connection_id)
        && connection.room_id.is_none()
    {
        connection.queued = false;
        server.waiting.retain(|id| *id != connection_id);
    }
}

async fn relay_signal(
    state: &AppState,
    connection_id: ConnectionId,
    message: ClientMessage,
    signal_type: &'static str,
) {
    let Some(room_id) = message.room_id else {
        return;
    };
    let Some(payload) = message.payload else {
        return;
    };

    let peer = {
        let server = state.inner.lock().await;
        server
            .rooms
            .get(&room_id)
            .filter(|room| room.participants.contains(&connection_id))
            .and_then(|room| room.participants.iter().find(|id| **id != connection_id))
            .and_then(|peer_id| server.connections.get(peer_id))
            .map(|connection| connection.outbound.clone())
    };

    if let Some(peer) = peer {
        let _ = peer.send(ServerMessage::new(
            signal_type,
            Some(&room_id),
            Some(payload),
        ));
    }
}

async fn open_decision(state: &AppState, room_id: &str) {
    let participants = {
        let mut server = state.inner.lock().await;
        let Some(room) = server.rooms.get_mut(room_id) else {
            return;
        };
        if room.phase != RoomPhase::AvatarDate {
            return;
        }
        room.phase = RoomPhase::Decision;
        participant_senders(&server, room_id)
    };

    let opened_at = unix_time_millis();
    let decision_seconds = state.decision_duration.as_secs();
    let message = ServerMessage::new(
        "decision.open",
        Some(room_id),
        Some(json!({
            "durationSeconds": decision_seconds,
            "openedAt": opened_at,
            "endsAt": opened_at + decision_seconds * 1_000
        })),
    );
    for outbound in participants {
        let _ = outbound.send(message.clone());
    }

    let timer_state = state.clone();
    let timer_room_id = room_id.to_owned();
    tokio::spawn(async move {
        tokio::time::sleep(timer_state.decision_duration).await;
        end_room_if_deciding(&timer_state, &timer_room_id, "decision_timeout").await;
    });
}

#[derive(Deserialize)]
struct DecisionPayload {
    decision: Decision,
}

async fn submit_decision(state: &AppState, connection_id: ConnectionId, message: ClientMessage) {
    let (room_id, decision) = match (message.room_id, message.payload) {
        (Some(room_id), Some(payload)) => {
            match serde_json::from_value::<DecisionPayload>(payload) {
                Ok(payload) => (room_id, payload.decision),
                Err(_) => {
                    send_to_connection(
                        state,
                        connection_id,
                        ServerMessage::error(
                            "invalid_decision",
                            "decision must be either reveal or end",
                        ),
                    )
                    .await;
                    return;
                }
            }
        }
        _ => return,
    };

    let result = {
        let mut server = state.inner.lock().await;
        let Some(room) = server.rooms.get_mut(&room_id) else {
            return;
        };
        if room.phase != RoomPhase::Decision
            || !room.participants.contains(&connection_id)
            || room.decisions.contains_key(&connection_id)
        {
            return;
        }
        room.decisions.insert(connection_id, decision);
        evaluate_decisions(room)
    };

    match result {
        DecisionResult::Pending => {}
        DecisionResult::Reveal => grant_reveal(state, &room_id).await,
        DecisionResult::End => end_room(state, &room_id, "decision_declined").await,
    }
}

fn evaluate_decisions(room: &Room) -> DecisionResult {
    if room
        .decisions
        .values()
        .any(|decision| *decision == Decision::End)
    {
        return DecisionResult::End;
    }
    if room
        .participants
        .iter()
        .all(|id| room.decisions.get(id) == Some(&Decision::Reveal))
    {
        return DecisionResult::Reveal;
    }
    DecisionResult::Pending
}

async fn grant_reveal(state: &AppState, room_id: &str) {
    let participants = {
        let mut server = state.inner.lock().await;
        let Some(room) = server.rooms.get_mut(room_id) else {
            return;
        };
        if room.phase != RoomPhase::Decision {
            return;
        }
        room.phase = RoomPhase::Revealed;
        participant_senders(&server, room_id)
    };
    let message = ServerMessage::new("reveal.granted", Some(room_id), None);
    for outbound in participants {
        let _ = outbound.send(message.clone());
    }
}

async fn end_room_if_deciding(state: &AppState, room_id: &str, reason: &'static str) {
    let participants = {
        let mut server = state.inner.lock().await;
        if !server
            .rooms
            .get(room_id)
            .is_some_and(|room| room.phase == RoomPhase::Decision)
        {
            return;
        }
        remove_room(&mut server, room_id)
    };
    let message = ServerMessage::new(
        "session.ended",
        Some(room_id),
        Some(json!({ "reason": reason })),
    );
    for outbound in participants {
        let _ = outbound.send(message.clone());
    }
}

async fn leave_session(state: &AppState, connection_id: ConnectionId, reason: &'static str) {
    let room_id = state
        .inner
        .lock()
        .await
        .connections
        .get(&connection_id)
        .and_then(|connection| connection.room_id.clone());
    if let Some(room_id) = room_id {
        end_room(state, &room_id, reason).await;
    }
}

async fn disconnect(state: &AppState, connection_id: ConnectionId) {
    let room_id = {
        let mut server = state.inner.lock().await;
        server.waiting.retain(|id| *id != connection_id);
        server
            .connections
            .remove(&connection_id)
            .and_then(|connection| connection.room_id)
    };
    if let Some(room_id) = room_id {
        end_room(state, &room_id, "peer_disconnected").await;
    }
}

async fn end_room(state: &AppState, room_id: &str, reason: &'static str) {
    let participants = {
        let mut server = state.inner.lock().await;
        remove_room(&mut server, room_id)
    };
    if participants.is_empty() {
        return;
    }
    let message = ServerMessage::new(
        "session.ended",
        Some(room_id),
        Some(json!({ "reason": reason })),
    );
    for outbound in participants {
        let _ = outbound.send(message.clone());
    }
}

fn remove_room(server: &mut ServerState, room_id: &str) -> Vec<Outbound> {
    let Some(room) = server.rooms.remove(room_id) else {
        return Vec::new();
    };
    room.participants
        .iter()
        .filter_map(|id| {
            server.connections.get_mut(id).map(|connection| {
                connection.room_id = None;
                connection.outbound.clone()
            })
        })
        .collect()
}

async fn send_to_connection(state: &AppState, connection_id: ConnectionId, message: ServerMessage) {
    let outbound = state
        .inner
        .lock()
        .await
        .connections
        .get(&connection_id)
        .map(|connection| connection.outbound.clone());
    if let Some(outbound) = outbound {
        let _ = outbound.send(message);
    }
}

fn participant_senders(server: &ServerState, room_id: &str) -> Vec<Outbound> {
    server
        .rooms
        .get(room_id)
        .into_iter()
        .flat_map(|room| room.participants)
        .filter_map(|id| server.connections.get(&id))
        .map(|connection| connection.outbound.clone())
        .collect()
}

fn duration_from_env(name: &str, default_seconds: u64) -> Result<Duration> {
    let seconds = env::var(name)
        .ok()
        .map(|value| value.parse())
        .transpose()
        .with_context(|| format!("{name} must be an integer"))?
        .unwrap_or(default_seconds);
    Ok(Duration::from_secs(seconds))
}

fn unix_time_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn add_connection(server: &mut ServerState, id: ConnectionId) {
        let (outbound, _receiver) = mpsc::unbounded_channel();
        server.connections.insert(
            id,
            Connection {
                outbound,
                queued: false,
                room_id: None,
            },
        );
    }

    #[test]
    fn pairs_two_waiting_connections_into_one_room() {
        let mut server = ServerState::default();
        add_connection(&mut server, 1);
        add_connection(&mut server, 2);

        assert!(server.join(1).is_none());
        let created = server.join(2).expect("the second connection should match");

        assert_eq!(created.participants[0].0, 1);
        assert_eq!(created.participants[1].0, 2);
        assert!(server.waiting.is_empty());
        assert_eq!(server.rooms.len(), 1);
        assert_eq!(
            server.connections[&1].room_id.as_deref(),
            Some(created.room_id.as_str())
        );
        assert_eq!(
            server.connections[&2].room_id.as_deref(),
            Some(created.room_id.as_str())
        );
    }

    #[test]
    fn chooses_randomly_from_multiple_waiting_connections() {
        let mut server = ServerState::default();
        for id in 1..=4 {
            add_connection(&mut server, id);
        }
        server.waiting.extend([1, 2, 3]);
        for id in [1, 2, 3] {
            server.connections.get_mut(&id).unwrap().queued = true;
        }

        let created = server.join(4).expect("a waiting connection should match");
        assert!([1, 2, 3].contains(&created.participants[0].0));
        assert_eq!(server.waiting.len(), 2);
    }

    #[test]
    fn only_two_reveal_decisions_grant_reveal() {
        let mut room = Room {
            participants: [1, 2],
            decisions: HashMap::new(),
            phase: RoomPhase::Decision,
        };
        assert_eq!(evaluate_decisions(&room), DecisionResult::Pending);

        room.decisions.insert(1, Decision::Reveal);
        assert_eq!(evaluate_decisions(&room), DecisionResult::Pending);

        room.decisions.insert(2, Decision::Reveal);
        assert_eq!(evaluate_decisions(&room), DecisionResult::Reveal);
    }

    #[test]
    fn any_end_decision_ends_the_room() {
        for decisions in [
            [(1, Decision::End), (2, Decision::Reveal)],
            [(1, Decision::Reveal), (2, Decision::End)],
            [(1, Decision::End), (2, Decision::End)],
        ] {
            let room = Room {
                participants: [1, 2],
                decisions: decisions.into_iter().collect(),
                phase: RoomPhase::Decision,
            };
            assert_eq!(evaluate_decisions(&room), DecisionResult::End);
        }
    }
}
