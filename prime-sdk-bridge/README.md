# prime-sdk-bridge

JSONL bridge from malvin (`prime:` models) to [`prime-agent`](https://github.com/PrimeIntellect-ai/prime-agent) via `createAgentSession`.

## Build

```bash
npm ci && npm run build
```

Requires Node ≥ 22.8. Dependency is pinned to the Prime release tarball (not the public npm registry).

## Protocol

Ops: `create`, `send`, `cancel`, `close` (no `resume` in v1).

Events: `ok`, `assistant`, `thinking`, `tool_call`, `usage`, `run_done`, `fatal`.

Default tools: `["ipython"]`. Auth via Prime `AuthStorage` / provider env keys — never Cursor credentials.
