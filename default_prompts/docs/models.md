# malvin admin models

List model ids for malvin runs. No malvin prompts and no run directory under `~/.malvin_home/logs/`.

## Summary

| | |
|---|---|
| Agent session | None |
| `.malvin/` | Not required |
| Output | Prefixed model list + `Current:` footer (see below) |

## Intention

Discover valid `--model` values for other malvin commands. Cursor SDK models use the `cursor:` prefix; Pi models use the `pi:` prefix from the linked `pi_agent_rust` registry; Codex models use the `codex:` prefix when an external `codex` binary is available.

## Usage

```text
malvin admin models [OPTION]... [PREFIX]...
```

## Global options

See `malvin --doc`. `admin` help does not list agent-session flags. Global `--model` before the subcommand (for example `malvin --model cursor:sonnet-4 admin models`) sets the `Current:` footer (overrides `~/.malvin_home/config.toml` when passed). Color follows the `NO_COLOR` environment variable.

Agent runs with `pi:` (like `cursor:` and `codex:`) always force tool auto-run. `--no-force` is not supported and fails fast before any session starts. `pi:` uses the linked `pi_agent_rust` crate and the operator’s Pi auth/config. Codex still requires a separate binary (`PATH` or `MALVIN_CODEX`; malvin does not bundle it) and a Codex login (`codex login`, `OPENAI_API_KEY`, or `$CODEX_HOME/auth.json`).

## Behavior

Listing:

1. List Cursor models via the Cursor SDK bridge (`Cursor.models.list`) when Node ≥ 22.13 and `cursor-sdk-bridge` are available (spawned through malvin’s sandbox command builder). Wall-clock budget defaults to 30s (`MALVIN_CURSOR_LIST_MODELS_TIMEOUT_MS`) for both the SDK Node path and the `agent` / `cursor-agent models` fallback. If that path fails, times out, or returns an empty catalog, fall back to `agent` / `cursor-agent models` on `PATH`. If both fail, print `(cursor models unavailable: …)` and continue with other sections. When a prefix filter cannot match any `cursor:` id, this section is skipped. If the SDK catalog omits `auto` (it may list `default` instead), malvin still prints `cursor:auto` so the documented CLI default remains discoverable.
2. Print each Cursor id with a `cursor:` prefix. When the SDK catalog includes parameter definitions, append a tab-separated summary such as `thinking=false|true effort=low|medium|high|xhigh|max fast=false|true` (parameter ids and allowed values vary by model; common ones are `thinking`, `effort` / `reasoning`, `fast`, and `context`).
3. List Pi models from the linked `pi_agent_rust` registry (`ModelRegistry::load_for_listing`), refreshing each authenticated provider’s live catalog at most once per day (cached under `~/.malvin_home/pi-model-cache/`). Use `--refresh` to force a live refetch. Keep only models whose provider you can run: an environment API key (from that provider’s Pi metadata) or a credential already stored in Pi’s auth file. Print each kept id with a `pi:` prefix (`pi:<provider>/<model>`), the model name, and `thinking=yes` or `thinking=no` when the registry reports reasoning. On failure, print `(pi models unavailable: …)` and continue. Skip when the prefix cannot match `pi:`.
4. List Codex models through the local stdio app-server (`codex app-server`, using `PATH` or `MALVIN_CODEX`) and print each with a `codex:` prefix, including hidden catalog ids. When the catalog includes them, append tab-separated `thinking=` reasoning levels, `service=` tier ids, `hidden`, and `default`. After the catalog rows, print family aliases that resolve at spawn (for example `codex:gpt-5.6` as `alias → gpt-5.6-sol` when two or more catalog ids share that prefix). If Codex is unavailable or the response cannot be parsed, print `(codex models unavailable: …)` and continue. Skip when the prefix cannot match `codex:`. Family names such as `codex:gpt-5.6` resolve at spawn to the first matching catalog variant (for example `gpt-5.6-sol`) when listing succeeds. Models that are not in the live catalog cannot be selected when listing succeeds.
5. Print blank line and: `Current: <model>` (from global `--model` when set, else `~/.malvin_home/config.toml`, else `cursor:auto`).

### Selecting thinking / speed

Bracket overrides use the same shape as the Cursor agent CLI (`id[k=v,…]`):

| Backend | Example | Effect |
|---------|---------|--------|
| `cursor:` | `cursor:claude-opus-5[thinking=true,effort=high,fast=true]` | Passed to the Cursor SDK as `{ id, params }` |
| `pi:` | `pi:openai/gpt-5[thinking=high]` | Passed to in-process `pi::sdk` as `SessionOptions.thinking` |
| `codex:` | `codex:gpt-5.6[thinking=high,service=priority]` | The slug is sent unchanged; `thinking` is sent as `turn/start.effort` and `service` as `serviceTier` |

For `pi:` and `codex:`, `thinking` uses the same levels: `off`, `minimal`, `low`, `medium`, `high`, `xhigh`, `max`, or `ultra`. Pi has no separate speed / `fast` switch. Capability `thinking=yes` in the listing means the model can use extended thinking; the bracket sets the level for a run. Vendors that do not name a level are mapped at the wire (`ultra` → Pi `max`; `off`/`minimal` → Codex `low`).

For `codex:`, the other supported bracket key is `service` (a catalog `service=` tier id such as `priority`). Unknown catalog slugs are rejected when `model/list` succeeds.

Optional trailing words form a **prefix filter** on printed model ids. Words are joined with `/` inserted between path segments when needed (so `malvin admin models pi: open` → `pi:open`). Examples: `malvin admin models pi:` lists only Pi models; `malvin admin models pi:open` lists Pi ids whose full id starts with `pi:open` (e.g. `pi:openai/…` and `pi:openrouter/…`). Catalog sections that cannot match the prefix are not queried.

`--refresh` forces a live refetch of Pi provider catalogs, bypassing the daily on-disk cache.

There is **no** `malvin admin models download` action. Local GGUF / `prime:` backends are not supported.

## Examples

```text
malvin admin models
malvin admin models cursor:
malvin admin models pi:
malvin admin models pi:open
malvin --model 'cursor:claude-opus-5[effort=high,fast=true]' do "say hi"
malvin --model 'pi:openai/gpt-5[thinking=high]' do "say hi"
malvin --model pi:openai/gpt-4o do "say hi"
malvin admin models codex:
malvin admin models --refresh pi:
malvin --model=codex:gpt-5.6 --do Hello
malvin --model='codex:gpt-5.6[thinking=high,service=priority]' --do Hello
malvin --model cursor:sonnet-4 admin models             # Current: footer shows cursor:sonnet-4
malvin --model cursor:sonnet-4 inspire plan.md
```
