# malvin models

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
malvin models [OPTION]... [PREFIX]...
```

## Global options

See `malvin --doc`. Global `--model` sets the `Current:` footer for this subcommand (overrides `~/.malvin_home/config.toml` when passed). Color follows the `NO_COLOR` environment variable.

Agent runs with `pi:` (like `cursor:`) always force tool auto-run. `--no-force` is not supported for `pi:` or `codex:` and fails fast. `pi:` uses the linked `pi_agent_rust` crate and the operator’s Pi auth/config. Codex still requires a separate binary (`PATH` or `MALVIN_CODEX`; malvin does not bundle it).

## Behavior

Listing:

1. List Cursor models via the Cursor SDK bridge (`Cursor.models.list`) when Node ≥ 22.13 and `cursor-sdk-bridge` are available (spawned through malvin’s sandbox command builder). Wall-clock budget defaults to 30s (`MALVIN_CURSOR_LIST_MODELS_TIMEOUT_MS`) for both the SDK Node path and the `agent` / `cursor-agent models` fallback. If that path fails, times out, or returns an empty catalog, fall back to `agent` / `cursor-agent models` on `PATH`. If both fail, print `(cursor models unavailable: …)` and continue with other sections. When a prefix filter cannot match any `cursor:` id, this section is skipped. If the SDK catalog omits `auto` (it may list `default` instead), malvin still prints `cursor:auto` so the documented CLI default remains discoverable.
2. Print each Cursor id with a `cursor:` prefix. When the SDK catalog includes parameter definitions, append a tab-separated summary such as `thinking=false|true effort=low|medium|high|xhigh|max fast=false|true` (parameter ids and allowed values vary by model; common ones are `thinking`, `effort` / `reasoning`, `fast`, and `context`).
3. List Pi models from the linked `pi_agent_rust` registry (`ModelRegistry::load_for_listing`) and keep only models whose provider has an env key or a stored Pi credential. Providers that list no auth-env keys (for example a local server) stay visible. Print each kept id with a `pi:` prefix (`pi:<provider>/<model>`), the model name, and `thinking=yes` or `thinking=no` when the registry reports reasoning. On failure, print `(pi models unavailable: …)` and continue. If the provider auth map cannot be built, print `(pi provider auth map unavailable: …)` and still print the `pi:` model rows unfiltered. Skip when the prefix cannot match `pi:`.
4. List Codex models through the local stdio app-server (`codex app-server`, using `PATH` or `MALVIN_CODEX`) and print each with a `codex:` prefix, including hidden catalog ids. When the catalog includes them, append tab-separated `thinking=` reasoning levels, `service=` tier ids, `hidden`, and `default`. If Codex is unavailable or the response cannot be parsed, print `(codex models unavailable: …)` and continue. Skip when the prefix cannot match `codex:`. Family aliases such as `codex:gpt-5.6` resolve at spawn time to the first live catalog id with that prefix (for example `gpt-5.6-sol`).
5. Print blank line and: `Current: <model>` (from global `--model` when set, else `~/.malvin_home/config.toml`, else `cursor:auto`).

### Selecting thinking / speed

Bracket overrides use the same shape as the Cursor agent CLI (`id[k=v,…]`):

| Backend | Example | Effect |
|---------|---------|--------|
| `cursor:` | `cursor:claude-opus-5[thinking=true,effort=high,fast=true]` | Passed to the Cursor SDK as `{ id, params }` |
| `pi:` | `pi:openai/gpt-5[thinking=high]` | Passed to in-process `pi::sdk` as `SessionOptions.thinking` |
| `codex:` | `codex:gpt-5.6[thinking=high,service=priority]` | Family names such as `gpt-5.6` map to the first matching catalog id; `thinking` is sent as `turn/start.effort` and `service` as `serviceTier` |

For `pi:`, the only supported bracket key is `thinking`, with values `off`, `minimal`, `low`, `medium`, `high`, `xhigh`, or `max`. Pi has no separate speed / `fast` switch. Capability `thinking=yes` in the listing means the model can use extended thinking; the bracket sets the level for a run.

For `codex:`, supported bracket keys are `thinking` (`low|medium|high|xhigh|max|ultra`, matching the catalog row) and `service` (a catalog `service=` tier id such as `priority`). Unknown catalog slugs are rejected when `model/list` succeeds.

Optional trailing words form a **prefix filter** on printed model ids. Words are joined with `/` inserted between path segments when needed (so `malvin models pi: open` → `pi:open`). Examples: `malvin models pi:` lists only Pi models; `malvin models pi:open` lists Pi ids whose full id starts with `pi:open` (e.g. `pi:openai/…` and `pi:openrouter/…`). Catalog sections that cannot match the prefix are not queried.

There is **no** `malvin models download` action. Local GGUF / `prime:` backends are not supported.

## Examples

```text
malvin models
malvin models cursor:
malvin models pi:
malvin models pi:open
malvin --model 'cursor:claude-opus-5[effort=high,fast=true]' do "say hi"
malvin --model 'pi:openai/gpt-5[thinking=high]' do "say hi"
malvin --model pi:openai/gpt-4o do "say hi"
malvin models codex:
malvin --model=codex:gpt-5.6 --do Hello
malvin --model='codex:gpt-5.6[thinking=high,service=priority]' --do Hello
malvin --model cursor:sonnet-4 models             # Current: footer shows cursor:sonnet-4
malvin --model cursor:sonnet-4 inspire plan.md
```
