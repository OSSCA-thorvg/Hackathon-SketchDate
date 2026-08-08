# SketchDate v0.0.1 PoC 구현 계획

## 목표

v0.0.1은 익명의 두 사용자가 다음 흐름을 끝까지 경험할 수 있는 최소한의 1:1 date app을 만든다.

1. 각자 캐릭터를 조합한다.
2. 카메라와 마이크를 준비하고 무작위 상대와 매칭된다.
3. 실제 얼굴 대신 표정을 따라 움직이는 캐릭터를 보며 음성으로 대화한다.
4. 대화가 끝나면 각자 얼굴 공개 여부를 비공개로 선택한다.
5. 두 사람 모두 동의한 경우에만 webcam video를 서로 공개하고 대화를 이어간다.

핵심 검증 대상은 **voice + animated character + mutual-consent video reveal**이다.

## 사용자 흐름

### 1. 캐릭터 만들기

- 현재 `front/`의 구현을 기준으로 `Head`, `Eyes`, `Nose`, `Mouth`, `Accessory`를 하나씩 선택한다.
- 각 선택은 즉시 ThorVG character preview에 반영한다.
- 기본 조합을 제공하므로 별도 편집 없이도 다음 단계로 갈 수 있다.
- 완료한 캐릭터는 현재 browser session 동안만 유지한다.

### 2. 미디어 준비

- camera와 microphone 권한을 한 번 요청한다.
- camera 영상은 MediaPipe 얼굴 추적 입력으로 사용하되 화면에는 표시하지 않는다.
- 캐릭터가 얼굴의 위치, 회전, 눈, 시선, 눈썹과 입 움직임을 따라가는지 보여준다.
- camera, microphone과 얼굴 감지가 준비되면 매칭을 시작할 수 있다.

### 3. 매칭

- 사용자는 익명 1:1 queue에 들어간다.
- 서버는 대기 중인 두 사용자를 무작위로 같은 room에 배정하고 WebRTC signaling 역할을 정한다.
- 별도 profile, 관심사, 성별/지역 filter와 계정은 사용하지 않는다.

### 4. 캐릭터 date

- 매칭 후 두 사용자의 microphone audio를 WebRTC로 연결한다.
- 화면에는 상대의 캐릭터가 상대의 얼굴 움직임에 맞춰 실시간으로 표시된다.
- local raw camera video는 계속 숨기며 WebRTC video sender에도 연결하지 않는다.
- microphone mute와 `연결 종료`만 항상 제공한다.
- 대화 주제 추천, 질문 card, 감정/표정 mission은 제공하지 않는다.
- PoC의 date round는 3분으로 하고, 종료 시 두 사용자를 비공개 결정 화면으로 이동한다.

### 5. 비공개 결정과 얼굴 공개

- 각 사용자는 `얼굴 공개` 또는 `대화 종료` 중 하나를 선택한다.
- 한 사용자의 선택은 상대에게 전송하거나 UI로 암시하지 않는다.
- 두 사용자 모두 `얼굴 공개`를 선택한 경우에만 서버가 양쪽에 reveal 승인을 보낸다.
- 한쪽이라도 `대화 종료`를 선택하거나 제한 시간 20초 안에 선택하지 않으면 양쪽 session을 동일한 종료 화면으로 끝낸다.
- reveal 승인을 받은 뒤에만 webcam track을 WebRTC video sender에 연결하고 재협상한다.
- remote video의 첫 frame이 준비되면 캐릭터 대신 video를 표시한다.
- 공개 후에도 microphone mute와 `연결 종료`를 유지한다.

## Frontend

### 페이지 상태

v0.0.1 frontend는 하나의 SvelteKit app 안에서 다음 상태만 관리한다.

```text
character → media-ready → matching → avatar-date → decision → video-date | ended
```

- router를 여러 페이지로 나눌 필요는 없으며 top-level state에 따라 화면을 교체해도 된다.
- active room 정보, media stream, peer connection과 선택한 캐릭터는 메모리에만 둔다.
- reload하면 진행 중인 session 복구를 시도하지 않고 처음으로 돌아간다.

### 캐릭터와 ThorVG

- 현재 frontend sketch의 `PartSelection`과 5개 part catalog를 v0.0.1 character format으로 사용한다.
- `ComposedAvatar.svelte`가 선택한 part를 SVG scene으로 조립한다.
- `CharacterRenderer.svelte`가 조립된 SVG와 pose를 `ThorVGRenderer.svelte`에 전달한다.
- `ThorVGRenderer.svelte`는 `@thorvg/webcanvas`로 live character를 canvas에 렌더링한다.
- part가 바뀔 때 SVG asset을 다시 만들고, 매 motion frame에는 `rig-*` paint의 transform/path만 갱신한다.
- picker의 작은 option preview는 DOM SVG를 사용해도 된다.
- 완성형 preset, 직접 그리기, 색상/transform 편집, undo/redo와 avatar document schema는 제외한다.

### 얼굴 추적과 motion

- `@mediapipe/tasks-vision` Face Landmarker를 `VIDEO` mode와 단일 얼굴 설정으로 사용한다.
- webcam frame에서 다음 transport-neutral `RigPose`를 만든다.

```ts
type RigPose = {
  x: number;
  y: number;
  scaleX: number;
  scaleY: number;
  rotation: number;
  yaw: number;
  pitch: number;
  leftEyeOpen: number;
  rightEyeOpen: number;
  leftGazeX: number;
  leftGazeY: number;
  rightGazeX: number;
  rightGazeY: number;
  leftBrowLift: number;
  rightBrowLift: number;
  mouthOpen: number;
  smile: number;
};
```

- landmark와 blendshape는 frontend 밖으로 보내지 않고 정규화한 `RigPose`만 전송한다.
- pose에 현재 sketch의 clamp와 단순 선형 smoothing을 적용한다.
- motion은 약 20Hz로 보내며 오래된 frame은 버린다.
- 얼굴을 찾지 못하면 neutral pose를 보내거나 상대 character를 neutral 상태로 되돌린다.
- 별도 calibration, Worker 추론, adaptive baseline과 고급 filter는 제외한다.

### WebRTC 연결

- microphone은 WebRTC audio track으로 전송한다.
- camera track은 로컬 얼굴 추적에만 사용하고 mutual reveal 전에는 peer connection에 추가하지 않는다.
- 초기 연결에 video transceiver를 만들되 `recvonly`로 둔다.
- server가 정한 initiator가 offer를 만들고 WebSocket을 통해 SDP/ICE를 relay한다.
- 두 DataChannel을 사용한다.
  - `avatar`: reliable/ordered. 연결 직후 `PartSelection`을 한 번 보내고 part가 바뀌면 최신 선택을 다시 보낸다.
  - `motion`: unordered, `maxRetransmits: 0`. 최신 `RigPose` snapshot만 보낸다.
- reveal 승인 후 양쪽이 camera track을 video transceiver sender에 연결하고 direction을 `sendrecv`로 바꾼다.
- 서버가 지정한 initiator가 한 번 더 offer를 만들어 video 연결을 협상한다.
- WebRTC 설정에는 TURN credential과 `iceTransportPolicy: "relay"`를 사용해 서로의 직접 IP 노출을 피한다.

### 최소 UI 상태

- model/camera/microphone 준비 중
- 얼굴을 찾는 중 / 얼굴 감지됨
- 상대를 찾는 중
- 연결 중
- avatar date와 남은 시간
- 비공개 선택 대기
- video 연결 중
- session 종료
- 계속 진행할 수 없는 오류가 발생하면 간단한 메시지와 `처음으로` action만 제공한다.

상세 오류 code, 자동 retry, reconnect/resume, renderer fallback과 복구별 UI는 v0.0.1에 포함하지 않는다.

## Backend

### 역할

- `back/`에 단일 Rust/Axum process를 구현한다.
- 상태는 database 없이 process memory에 저장한다.
- backend는 다음만 담당한다.
  - 대기 queue와 1:1 room 생성
  - initiator 지정
  - SDP/ICE signaling relay
  - 3분 avatar date 및 20초 decision timer
  - 비공개 reveal 결정 판정
  - Cloudflare TURN credential 발급
  - leave/disconnect 시 room 종료
- audio, video, character와 motion payload는 backend를 거치지 않는다.

### Endpoint

- `GET /healthz`: 배포 확인용 `200` response.
- `GET /ws`: matchmaking, signaling, timer, decision과 session 종료 event를 처리하는 WebSocket.

### 최소 message 목록

Client → server:

- `match.join`
- `match.cancel`
- `signal.offer`
- `signal.answer`
- `signal.ice`
- `decision.submit`
- `session.leave`

Server → client:

- `match.queued`
- `match.found`
- `turn.credentials`
- `signal.offer`
- `signal.answer`
- `signal.ice`
- `round.started`
- `decision.open`
- `reveal.granted`
- `session.ended`
- `server.error`

- 모든 message는 `v`, `type`과 필요한 경우 `roomId`, `payload`를 가진 작은 JSON object로 통일한다.
- 서버는 room 참가자가 아닌 connection의 signaling과 decision message를 무시한다.
- reconnect, resume token, request deduplication, message buffering과 복잡한 error taxonomy는 구현하지 않는다.

### Decision 규칙

- 선택 값은 `reveal` 또는 `end` 두 개뿐이다.
- 선택은 제출 후 변경하지 않는다.
- 서버는 두 선택을 모두 받기 전까지 개별 선택을 상대에게 알리지 않는다.
- 결과는 다음 두 가지뿐이다.
  - `reveal + reveal` → 양쪽에 `reveal.granted`
  - 그 외 조합 또는 timeout → 양쪽에 동일한 `session.ended`

## 배포

- frontend는 SvelteKit static build로 GitHub Pages에 배포한다.
- backend는 단일 Render Web Service에 배포한다.
- TURN은 Cloudflare Realtime TURN을 사용하고 credential은 backend가 발급한다.
- frontend에는 backend의 HTTPS/WSS public URL만 설정한다.
- TURN API token은 backend environment secret으로만 저장한다.
- production frontend origin과 local development origin만 backend에서 허용한다.
- database, persistent disk, Redis, 별도 worker와 수평 확장은 사용하지 않는다.
- 완전한 offline asset bundle은 필수가 아니다. 현재 sketch처럼 MediaPipe WASM/model을 public asset URL에서 load해도 된다.

## 구현 순서

1. 현재 character picker, MediaPipe pose와 ThorVG rendering을 하나의 local flow로 정리한다.
2. character/media/matching/date/decision/video/ended 화면 상태를 연결한다.
3. Axum WebSocket queue, room과 signaling relay를 구현한다.
4. WebRTC audio와 `avatar`/`motion` DataChannel을 연결한다.
5. avatar date timer와 private decision 판정을 구현한다.
6. mutual reveal 뒤 camera track 추가와 video 재협상을 구현한다.
7. TURN credential 발급과 relay-only WebRTC 설정을 연결한다.
8. frontend/backend를 배포하고 실제 두 device에서 전체 flow를 확인한다.

## 검증 및 완료 기준

### 자동 확인

- frontend: `npm run check`, `npm run lint`, `npm run build`.
- backend: `cargo fmt --check`, `cargo clippy`, `cargo test`, release build.
- backend unit test는 queue pairing과 decision 조합만 최소로 포함한다.

### 수동 end-to-end 확인

- 서로 다른 두 browser/device가 한 room에 매칭된다.
- 두 사용자가 상대의 목소리를 듣고 mute/unmute할 수 있다.
- 상대가 선택한 character가 정확히 표시된다.
- 상대의 머리, 눈, 시선, 눈썹과 입 움직임이 character에 반영된다.
- avatar date 동안 어떤 raw camera video도 표시되거나 전송되지 않는다.
- 한 명만 reveal을 선택하면 video가 공개되지 않고 상대는 그 선택을 알 수 없다.
- 두 명 모두 reveal을 선택하면 그 이후에만 양쪽 webcam video가 표시된다.
- 어느 단계에서든 `연결 종료`를 누르면 양쪽 session이 종료되고 media track과 connection이 정리된다.
- 배포된 frontend와 backend에서 위 흐름이 동작한다.

## 명시적 비범위

- 계정, profile, 연령 인증과 사용자 검색/filter
- text chat, 연락처 교환과 친구 기능
- 대화 주제, 질문 card와 감정/표정 mission
- drawing editor, avatar 저장과 여러 preset
- 녹화, screenshot, analytics와 moderation 도구
- reconnect/session resume와 새로고침 복구
- 상세 오류 처리, compatibility matrix와 성능 최적화
- 신고/차단, 운영자 console과 production-scale abuse prevention
- mobile 전용 UX와 다국어

신고/차단과 abuse prevention은 실제 공개 서비스 전에 반드시 필요하지만, 제한된 v0.0.1 PoC 구현 범위에서는 제외한다.
