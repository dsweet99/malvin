# cursor-sdk-bridge

Node ≥ 22.13 sidecar that speaks malvin's JSONL bridge protocol and drives
`@cursor/sdk` (`Agent.create` → `agent.send` → `run.stream` / `run.wait`).

## Dev

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
