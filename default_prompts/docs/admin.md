# malvin admin

Operator maintenance commands. No agent session and no run directory under `~/.malvinconf/logs/`.

## Summary

| | |
|---|---|
| Agent session | None |
| `.malvin/` | Not required |
| Output | Short status line on success (or model list for `models`) |

## Intention

Fix local malvin/herdr bookkeeping, install the Cursor SDK, or list available model ids, without starting a research or coding turn. Agent-session flags (`--model`, `-g`, …) are not listed on `admin` help; see `malvin --doc`.

## Usage

```text
malvin admin <COMMAND>
malvin admin models [OPTION]... [PREFIX]...
malvin admin reset-herdr
malvin admin rh
malvin admin setup-cursor
```

## Subcommands

### `models`

List `cursor:`, `pi:`, `rpi:`, and `codex:` model ids. See `malvin admin models --doc` for the full contract.

### `reset-herdr`

Alias: `rh`.

Set the current herdr pane's malvin agent lifecycle state to idle (not working) and clear display metadata.

Requires a herdr-hosted environment: `HERDR_ENV=1`, `HERDR_SOCKET_PATH`, and `HERDR_PANE_ID`. Useful when a prior malvin process exited without tearing down and the pane still shows `working`.

### `setup-cursor`

Install the Cursor SDK (`@cursor/sdk`) that `cursor:` models need. Malvin writes its bundled bridge files to `~/.malvinconf/sdk-bridges/cursor-sdk-bridge/` and runs `npm ci --omit=dev` there. Requires Node.js ≥ 22.13, `npm`, and network access to the npm registry. Running it again is cheap: npm runs only when the SDK is missing or the bundled lock file changed. Malvin also does this automatically the first time a `cursor:` model runs; use this command to install ahead of time or to see errors directly.
