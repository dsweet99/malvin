# malvin models

List model ids for malvin runs. No malvin prompts and no run directory under `~/.malvin_home/logs/`.

## Summary

| | |
|---|---|
| Agent session | None |
| `.malvin/` | Not required |
| Output | Prefixed model list + `Current:` footer (see below) |

## Intention

Discover valid `--model` values for other malvin commands. Cursor ACP models use the `cursor:` prefix; OpenRouter (malvin-mini) models use the `openrouter:` prefix; local GGUF models (in-process llama.cpp / Metal) use the `local:` prefix.

## Usage

```text
malvin models [OPTIONS]
malvin models download local:<id>
```

## Global options

See `malvin --doc`. Only `--no-color` materially affects output formatting. Global `--model` is parsed but **not used** by this subcommand. `--no-download` applies when running agent commands with `local:` models (not this listing).

## Behavior

1. Resolve `agent` or `cursor-agent` on `PATH`.
2. Run `<binary> models` and print each id with a `cursor:` prefix.
3. Fetch OpenRouter models (when available) and print each id with an `openrouter:` prefix.
4. Print built-in `local:` models (no cache-status suffix) only when this build supports Apple Silicon Metal (the only local GPU backend). Otherwise omit the `local:` section entirely.
5. Print blank line and: `Current: <model>` (from `~/.malvin_home/config.toml`, else `cursor:auto`).

`malvin models download local:<id>` fetches a GGUF into `~/.malvin_home/model_cache/` (Apple Silicon / Metal). Known v1 ids: `local:qwen35_9b_q4`, `local:nemotron3_nano_4b`. On hosts without Metal, those ids are not shown by `malvin models` even though download may still be requested explicitly.

`local:` models run **in-process** under the agent sandbox USS cap. Before load, malvin requires `mem_limit_gb` in `~/.malvin_home/config.toml` to meet the model floor (Nano ≥ 6, Qwen ≥ 8). The default template is `4`, which is too small for either model — raise it first or `ensure_local_engine` fails with a clear error.

Context window defaults to `context_size = 8192` in `~/.malvin_home/config.toml` (llama.cpp `n_ctx` / `n_ctx_seq`). Raise it for longer prompts; larger windows need more `mem_limit_gb` headroom. Prompts that tokenize to ≥ `context_size` tokens fail fast with a clear error. Rebuild/install from this workspace (`cargo install --path .`) if your PATH `malvin` still lists MLX / Cascade2 ids.

Root `kiss check` ignores `malvin-llama/` (see `.kissignore`); structural cleanliness is verified with `cd malvin-llama && kiss check` (local threshold). Qwen end-to-end smoke is optional (same engine path as Nano).

## Examples

```text
malvin models
malvin --no-color models
malvin models download local:qwen35_9b_q4
malvin --model local:qwen35_9b_q4 do "say hi"
malvin --model cursor:sonnet-4 inspire plan.md    # --model applies to agent subcommands, not models
```
