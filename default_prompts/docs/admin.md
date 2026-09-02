# malvin admin

Operator maintenance commands. No agent session and no run directory under `~/.malvin_home/logs/`.

## Summary

| | |
|---|---|
| Agent session | None |
| `.malvin/` | Not required |
| Output | Short status line on success (or model list for `models`) |

## Intention

Fix local malvin/herdr bookkeeping, or list available model ids, without starting a research or coding turn. Agent-session flags (`-b`, `--model`, `-g`, …) are not listed on `admin` help; see `malvin --doc`.

## Usage

```text
malvin admin <COMMAND>
malvin admin models [OPTION]... [PREFIX]...
malvin admin reset-herdr
```

## Subcommands

### `models`

List `cursor:`, `pi:`, and `codex:` model ids. See `malvin admin models --doc` for the full contract.

### `reset-herdr`

Set the current herdr pane's malvin agent lifecycle state to idle (not working) and clear display metadata.

Requires a herdr-hosted environment: `HERDR_ENV=1`, `HERDR_SOCKET_PATH`, and `HERDR_PANE_ID`. Useful when a prior malvin process exited without tearing down and the pane still shows `working`.
