# SketchDate backend

Single-process Axum signaling server for the v0.0.1 proof of concept.

## Run locally

```sh
TURN_KEY_ID=your-cloudflare-turn-key-id \
TURN_KEY_API_TOKEN=your-cloudflare-turn-key-token \
cargo run
```

The server listens on `0.0.0.0:3000` by default and exposes:

- `GET /healthz`
- `GET /ws` (WebSocket upgrade)

## Configuration

| Variable | Default | Purpose |
| --- | --- | --- |
| `PORT` | `3000` | HTTP listen port |
| `ALLOWED_ORIGINS` | local Svelte dev/preview origins | Comma-separated browser origins allowed to open `/ws` |
| `AVATAR_ROUND_SECONDS` | `180` | Avatar date duration |
| `DECISION_SECONDS` | `20` | Private decision duration |
| `TURN_KEY_ID` | none | Cloudflare Realtime TURN key ID |
| `TURN_KEY_API_TOKEN` | none | Secret token belonging to the TURN key |
| `TURN_TTL_SECONDS` | `3600` | Lifetime of issued ICE server credentials |

Set `ALLOWED_ORIGINS` to the deployed frontend origin in production. Missing TURN
configuration does not prevent the service from starting, but matched clients receive a
`server.error` with code `turn_unavailable` instead of `turn.credentials`.

## Protocol

Every message is JSON with `v: 1`, `type`, optional `roomId`, and optional `payload`.
The supported message names match `PLAN.md`. A `decision.submit` payload is shaped as:

```json
{ "decision": "reveal" }
```

Signal payloads are opaque JSON and are relayed only to the other participant in the
specified room. The server never receives character, motion, audio, or video data.
