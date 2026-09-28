# cursor-sdk-bridge

Node ≥ 22.13 sidecar that speaks malvin's JSONL bridge protocol and drives
`@cursor/sdk` (`Agent.create` → `agent.send` → `run.stream` / `run.wait`).

## Install

The malvin binary embeds `package.json`, `package-lock.json`, and the non-test
`dist/*.js` files. At run time (first `cursor:` use, or
`malvin admin setup-cursor`) it writes them to
`~/.malvin_home/sdk-bridges/cursor-sdk-bridge/` and runs `npm ci --omit=dev`
there. The build does not run npm, so commit `dist/` after changing `src/`.
For a manual in-tree rebuild:

```bash
cd cursor-sdk-bridge
npm ci
npm run build
```

Entry points:

- `node dist/bridge.js` — long-lived session bridge (stdin/stdout JSONL)
- `node dist/models.js` — one-shot `cursor:` model listing

Malvin resolves the bridge relative to the repo / install prefix, or via
`MALVIN_CURSOR_SDK_BRIDGE`.
