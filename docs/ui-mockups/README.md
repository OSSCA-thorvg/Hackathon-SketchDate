# SketchDate UI Mockups

Implementation reference for the desktop-only SketchDate proof of concept described in `PLAN.md`.

## Shared visual system

- **Viewport:** 1600×1000 reference frame (16:10), usable down to 1024×768.
- **Personality:** warm, candid, playful, and safe; hand-drawn avatar artwork inside clean product UI.
- **Colors:** cream `#FFF9F2`, paper `#FFFFFF`, ink `#252525`, coral `#FF625F`, butter `#FFC857`, teal `#78C8C5`, sky `#77B7E5`, lavender `#AD99CF`, success `#28A96B`.
- **Surfaces:** 16–20px rounded cards, 1px warm-gray borders, restrained soft shadows.
- **Typography:** readable Korean sans-serif for controls; hand-lettered treatment only for large emotional headings.
- **Controls:** minimum 44px targets, icon plus visible Korean label for consequential actions.
- **Avatar rule:** avatar surfaces and thumbnails represent ThorVG output. UI chrome represents Svelte/HTML/CSS.
- **Privacy rule:** no camera video is visible until mutual reveal succeeds.
- **Persistent call controls:** microphone mute and `연결 종료` remain visible throughout avatar and revealed calls.

## Screen inventory

| File | State | Primary implementation purpose |
| --- | --- | --- |
| `01-landing-server-wake.png` | Landing / backend wake | Explain the product and visibly handle Render cold start. |
| `02-character-editor.png` | Character editor | Presets, drawing, semantic parts, palette, transforms, autosave. |
| `03-media-readiness.png` | Camera/microphone/face readiness | Confirm permissions, active devices, and face tracking without displaying raw video. |
| `04-matchmaking.png` | Queue | Cancellable matching progress with saved-avatar preview. |
| `05-secure-connection.png` | Match found / setup | Avatar exchange, relay verification, and connection preparation. |
| `06-avatar-round-question.png` | Avatar call / question card | Main voice conversation layout and synchronized round timer. |
| `07-avatar-round-mission.png` | Avatar call / expression mission | Mission-card variant and tracking feedback. |
| `08-private-decision.png` | 20-second private decision | `얼굴 공개` versus `대화 종료` without revealing peer choice. |
| `09-reveal-connecting.png` | Mutual reveal transition | Video negotiation/first-frame wait and reveal progress. |
| `10-revealed-video-call.png` | Revealed video call | Video conversation after mutual consent. |
| `11-session-ended.png` | Neutral terminal state | Generic end messaging and safe return to the saved editor. |
| `12-system-states.png` | Error/edge-state reference sheet | Permission, tracking, reconnect, compatibility, busy, and unsupported-layout patterns. |
| `13-interaction-states.png` | Interaction/modal reference sheet | Reset, invalid saved data, mute variants, submitted choice, disabled choice, and restart behavior. |

`00-overview-contact-sheet.png` is the visual index of the canonical set. Superseded exploratory images are retained under `_drafts/` and are not implementation references.

## Consistency invariants

1. The heart-and-pencil mark, top bar, cream background, ink drawing style, coral primary action, and card geometry remain consistent.
2. Before reveal, people are represented only by their avatars; no raw camera preview appears.
3. Selection or timeout outcomes never disclose the other participant's private choice.
4. Errors use plain Korean, one clear recovery action where recovery is allowed, and no internal diagnostic details.
5. Progress screens always explain what is happening and provide cancellation when cancellation is safe.
6. Exact implementation copy is governed by `PLAN.md` and this index; generated raster text is a visual reference, not a localization source file.
