# SketchDate frontend

SvelteKit/Tailwind frontend for the SketchDate v0.0.1 proof of concept. The app keeps the
entire session in memory and follows the flow in the repository's `PLAN.md`:

`landing → media-ready → matching → avatar-date → decision → video-date | ended`

## Local development

Copy the example environment file and point it at the Axum backend when needed:

```sh
cp .env.example .env
npm install
npm run dev
```

The local default is `ws://localhost:3000/ws`, so the environment variable can be omitted
when the backend uses its default port.

## Character and media pipeline

- MediaPipe Face Landmarker runs locally and turns camera landmarks/blendshapes into the
  transport-neutral `RigPose` shape.
- `CharacterRenderer.svelte` builds an SVG asset only when the selected character changes.
- `ThorVGRenderer.svelte` loads that asset through `@thorvg/webcanvas` and updates named rig
  paints for each motion frame without reparsing the character.
- WebRTC sends microphone audio, a reliable character-selection channel, and an unordered
  20 Hz motion channel.
- The camera track is never attached to WebRTC until the backend sends `reveal.granted`.

## Validation

```sh
npm run check
npm run lint
npm run build
```

## GitHub Pages deployment

Push the repository to `OSSCA-thorvg/Hackathon-SketchDate`. The workflow at
`.github/workflows/deploy-pages.yml` checks, builds, and deploys this frontend to:

`https://ossca-thorvg.github.io/Hackathon-SketchDate/`

In the GitHub repository, open **Settings → Pages** and set **Source** to
**GitHub Actions**. For matchmaking to work in production, also add an Actions repository
variable named `PUBLIC_BACKEND_WS_URL` under **Settings → Secrets and variables → Actions**.
Its value must be the backend's public secure WebSocket URL, for example:

`wss://api.example.com/ws`

The URL is compiled into the browser bundle and is therefore public; use a repository
variable rather than a secret. If the variable is omitted, the site still builds and renders,
but matchmaking falls back to `wss://ossca-thorvg.github.io:3000/ws` and will not connect
unless a compatible backend is actually available there.

Production builds use `/Hackathon-SketchDate` as their default base path. Override it with
the `BASE_PATH` environment variable when building for another Pages repository or a custom
domain (use an empty value for the domain root).
