# malvin models

List model ids for malvin runs. No malvin prompts and no run directory under `~/.malvin_home/logs/`.

## Summary

| | |
|---|---|
| Agent session | None |
| `.malvin/` | Not required |
| Output | Prefixed model list + `Current:` footer (see below) |

## Intention

Discover valid `--model` values for other malvin commands. Cursor SDK models use the `cursor:` prefix; Prime SDK models use the `prime:` prefix; OpenRouter (malvin-mini) models use `mini:openrouter/…`; local GGUF models use `mini:local/…`.

## Usage

```text
malvin models [OPTIONS]
malvin models [PREFIX]...
```

## Global options

See `malvin --doc`. Global `--model` is parsed but **not used** by this subcommand. `--no-download` applies when running agent commands with `mini:local/…` models (not this listing). Color follows the `NO_COLOR` environment variable.

## Behavior

Listing:

1. List Cursor models via the Cursor SDK bridge (`Cursor.models.list`) when Node ≥ 22.13 and `cursor-sdk-bridge` are available. If that path fails, fall back to `agent` / `cursor-agent models` on `PATH`. If both fail, the Cursor section errors (Prime / Mini sections may still print depending on caller flow). When a prefix filter cannot match any `cursor:` id, this section is skipped.
2. Print each Cursor id with a `cursor:` prefix.
3. List Prime models via the Prime SDK bridge (`models.js`) when available; otherwise fall back to `prime-agent model list` on `PATH`. Print each id with a `prime:` prefix (full catalog; no truncation hint). On failure print `(prime models unavailable: …)` and continue. Skip when the prefix cannot match `prime:`.
4. Fetch OpenRouter models (best-effort when the API key / network is available) and print each id with a `mini:openrouter/` prefix; on failure print `(mini:openrouter models unavailable: …)` and continue. Skip when the prefix cannot match that head.
5. Print built-in `mini:local/…` models (no cache-status suffix) only when this build supports Apple Silicon Metal. Otherwise omit the local section entirely.
6. Print blank line and: `Current: <model>` (from `~/.malvin_home/config.toml`, else `cursor:auto`).

Optional trailing words form a **prefix filter** on printed model ids. Words are concatenated with no separator (so `malvin models prime: open` is the same as `malvin models prime:open`). Examples: `malvin models prime:` lists only Prime models; `malvin models prime:open` lists Prime ids whose full id starts with `prime:open` (e.g. `prime:openai/…` and `prime:openrouter/…`). Catalog sections that cannot match the prefix are not queried.

There is **no** `malvin models download` action. `mini:local/…` GGUF files are fetched automatically into `~/.malvin_home/model_cache/` on first use unless `--no-download` is set. Known v1 ids: `mini:local/qwen35_9b_q4`, `mini:local/nemotron3_nano_4b`. On hosts without Metal, those ids are not shown by `malvin models`.

`mini:local/…` models run **in-process** under the agent sandbox USS cap. Before load, malvin requires `mem_limit_gb` in `~/.malvin_home/config.toml` to meet the model floor (Nano ≥ 6, Qwen ≥ 8). The default template is `4`, which is too small for either model — raise it first or `ensure_local_engine` fails with a clear error.

Context window defaults to `context_size = 8192` in `~/.malvin_home/config.toml` (llama.cpp `n_ctx` / `n_ctx_seq`). Raise it for longer prompts; larger windows need more `mem_limit_gb` headroom. Prompts that tokenize to ≥ `context_size` tokens fail fast with a clear error. Rebuild/install from this workspace (`cargo install --path .`) if your PATH `malvin` still lists MLX / Cascade2 ids.

Local llama.cpp integration lives in `src/malvin_llama/` (ignored by root `kiss check`; see `.kissignore`). Qwen end-to-end smoke is optional (same engine path as Nano).

## Live transport and agent-backend tests

Default `cargo nextest run` skips network and GPU paths. Opt-in live suites use `#[ignore]` plus env gates. When a live gate is **set**, missing prereqs fail the test (assert / fail-closed), not a soft skip. When a gate is **unset**, ignored bodies may return early without running live work.

| Env | What it enables |
|---|---|
| `MALVIN_LIVE_TRANSPORT=1` + `OPENROUTER_API_KEY` | `LlmTransport::OpenRouter` live `ensure_ready` + short `complete` (`tests/transport_live.rs`) |
| `MALVIN_LIVE_LOCAL=1` | Real local/GPU `LlmTransport::Local` and Mini+`mini:local/…` via `AgentBackend` (`tests/transport_live.rs`, `tests/agent_backend_live.rs`). **Metal / Apple Silicon only**; leave unset on hosts without a GPU. |
| `MALVIN_LIVE_MINI=1` + `OPENROUTER_API_KEY` | Mini+OpenRouter via `AgentBackend` API (`tests/agent_backend_live.rs`) and existing CLI live suite (`tests/mini_live.rs`) |

Live Cursor agent-backend cases reuse the existing live-agent prereqs in `tests/common/live_agent.rs` (no new `MALVIN_LIVE_AGENT`).

```text
MALVIN_LIVE_TRANSPORT=1 OPENROUTER_API_KEY=... cargo nextest run -E 'test(transport_live)' -- --ignored
MALVIN_LIVE_LOCAL=1 cargo nextest run -E 'test(transport_live)' -- --ignored
MALVIN_LIVE_MINI=1 OPENROUTER_API_KEY=... cargo nextest run -E 'test(agent_backend_live)' -- --ignored
MALVIN_LIVE_LOCAL=1 cargo nextest run -E 'test(agent_backend_live)' -- --ignored
MALVIN_LIVE_MINI=1 cargo nextest run mini_live -- --ignored
```

## Examples

```text
malvin models
malvin models prime:
malvin models prime:open
malvin models mini:local/
malvin --model mini:local/qwen35_9b_q4 do "say hi"
malvin --model cursor:sonnet-4 inspire plan.md    # --model applies to agent subcommands, not models
```
