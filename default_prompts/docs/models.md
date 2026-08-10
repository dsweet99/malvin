# malvin models

List model ids for malvin runs. No malvin prompts and no run directory under `~/.malvin_home/logs/`.

## Summary

| | |
|---|---|
| Agent session | None |
| `.malvin/` | Not required |
| Output | Prefixed model list + `Current:` footer (see below) |

## Intention

Discover valid `--model` values for other malvin commands. Cursor SDK models use the `cursor:` prefix; Prime SDK models use the `prime:` prefix (including `prime:local/…` for the built-in GGUF catalog, served via an in-process OpenAI-compatible sidecar).

## Usage

```text
malvin models [OPTIONS]
malvin models [PREFIX]...
```

## Global options

See `malvin --doc`. Global `--model` is parsed but **not used** by this subcommand. `--no-download` applies when running agent commands with `prime:local/…` models (not this listing). Color follows the `NO_COLOR` environment variable.

## Behavior

Listing:

1. List Cursor models via the Cursor SDK bridge (`Cursor.models.list`) when Node ≥ 22.13 and `cursor-sdk-bridge` are available. If that path fails, fall back to `agent` / `cursor-agent models` on `PATH`. If both fail, print `(cursor models unavailable: …)` and continue with other sections. When a prefix filter cannot match any `cursor:` id, this section is skipped. If the SDK catalog omits `auto` (it may list `default` instead), malvin still prints `cursor:auto` so the documented CLI default remains discoverable.
2. Print each Cursor id with a `cursor:` prefix.
3. List Prime models via the Prime SDK bridge (`models.js`) when available; otherwise fall back to `prime-agent model list` on `PATH`. Print each id with a `prime:` prefix (full catalog; no truncation hint). On failure print `(prime models unavailable: …)` and continue. Skip when the prefix cannot match `prime:`.
4. When listing Prime models on Apple Silicon Metal hosts, also print the built-in GGUF catalog as `prime:local/…` (no cache-status suffix). Otherwise omit local ids.
5. Print blank line and: `Current: <model>` (from `~/.malvin_home/config.toml`, else `cursor:auto`).

Optional trailing words form a **prefix filter** on printed model ids. Words are joined with `/` inserted between path segments when needed (so `malvin models prime: open` → `prime:open`). Examples: `malvin models prime:` lists only Prime models; `malvin models prime:open` lists Prime ids whose full id starts with `prime:open` (e.g. `prime:openai/…` and `prime:openrouter/…`). Catalog sections that cannot match the prefix are not queried.

There is **no** `malvin models download` action. Local GGUF files are fetched automatically into `~/.malvin_home/model_cache/` on first use unless `--no-download` is set. Known v1 ids: `prime:local/qwen35_9b_q4`, `prime:local/nemotron3_nano_4b`. On hosts without Metal, those ids are not shown by `malvin models`.

`prime:local/…` loads the GGUF and exposes it to prime-agent over a localhost OpenAI-compatible sidecar (still under the agent sandbox USS cap). Before load, malvin requires `mem_limit_gb` in `~/.malvin_home/config.toml` to meet the model floor (Nano ≥ 6, Qwen ≥ 8). The default template is `4`, which is too small for either model — raise it first or `ensure_local_engine` fails with a clear error.

Context window defaults to `context_size = 8192` in `~/.malvin_home/config.toml` (llama.cpp `n_ctx` / `n_ctx_seq`). Raise it for longer prompts; larger windows need more `mem_limit_gb` headroom. Prompts that tokenize to ≥ `context_size` tokens fail fast with a clear error. Rebuild/install from this workspace (`cargo install --path .`) if your PATH `malvin` still lists obsolete ids.

Local llama.cpp integration lives in `src/malvin_llama/` (ignored by root `kiss check`; see `.kissignore`).

## Examples

```text
malvin models
malvin models prime:
malvin models prime:open
malvin models prime:local/
malvin --model prime:local/qwen35_9b_q4 do "say hi"
malvin --model cursor:sonnet-4 inspire plan.md    # --model applies to agent subcommands, not models
```
