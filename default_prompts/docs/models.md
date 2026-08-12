# malvin models

List model ids for malvin runs. No malvin prompts and no run directory under `~/.malvin_home/logs/`.

## Summary

| | |
|---|---|
| Agent session | None |
| `.malvin/` | Not required |
| Output | Prefixed model list + `Current:` footer (see below) |

## Intention

Discover valid `--model` values for other malvin commands. Cursor SDK models use the `cursor:` prefix; Pi models use the `pi:` prefix when an external `pi` binary is available.

## Usage

```text
malvin models [OPTIONS]
malvin models [PREFIX]...
```

## Global options

See `malvin --doc`. Global `--model` sets the `Current:` footer for this subcommand (overrides `~/.malvin_home/config.toml` when passed). Color follows the `NO_COLOR` environment variable.

Agent runs with `pi:` (like `cursor:`) always force tool auto-run. `--no-force` is not supported for `pi:` and fails fast; install `pi` separately (`PATH` or `MALVIN_PI`; malvin does not bundle it). Minimum tested CLI: `pi 0.1.23`.

## Behavior

Listing:

1. List Cursor models via the Cursor SDK bridge (`Cursor.models.list`) when Node ≥ 22.13 and `cursor-sdk-bridge` are available (spawned through malvin’s sandbox command builder). Wall-clock budget defaults to 30s (`MALVIN_CURSOR_LIST_MODELS_TIMEOUT_MS`) for both the SDK Node path and the `agent` / `cursor-agent models` fallback. If that path fails, times out, or returns an empty catalog, fall back to `agent` / `cursor-agent models` on `PATH`. If both fail, print `(cursor models unavailable: …)` and continue with other sections. When a prefix filter cannot match any `cursor:` id, this section is skipped. If the SDK catalog omits `auto` (it may list `default` instead), malvin still prints `cursor:auto` so the documented CLI default remains discoverable.
2. Print each Cursor id with a `cursor:` prefix.
3. List Pi models via `pi --list-models` when `pi` is on `PATH` or `MALVIN_PI` is set (spawned through malvin’s sandbox command builder). Wall-clock budget defaults to 30s (`MALVIN_PI_LIST_MODELS_TIMEOUT_MS`). Print each id with a `pi:` prefix (`pi:<provider>/<model>`). On failure, timeout, or exit 0 with no parseable rows, print `(pi models unavailable: …)` and continue. Skip when the prefix cannot match `pi:`.
4. Print blank line and: `Current: <model>` (from global `--model` when set, else `~/.malvin_home/config.toml`, else `cursor:auto`).

Optional trailing words form a **prefix filter** on printed model ids. Words are joined with `/` inserted between path segments when needed (so `malvin models pi: open` → `pi:open`). Examples: `malvin models pi:` lists only Pi models; `malvin models pi:open` lists Pi ids whose full id starts with `pi:open` (e.g. `pi:openai/…` and `pi:openrouter/…`). Catalog sections that cannot match the prefix are not queried.

There is **no** `malvin models download` action. Local GGUF / `prime:` backends are not supported.

## Examples

```text
malvin models
malvin models cursor:
malvin models pi:
malvin models pi:open
malvin --model pi:openai/gpt-4o do "say hi"
malvin --model cursor:sonnet-4 models             # Current: footer shows cursor:sonnet-4
malvin --model cursor:sonnet-4 inspire plan.md
```
