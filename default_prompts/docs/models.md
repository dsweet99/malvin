# malvin models

List model ids for malvin runs. No malvin prompts and no run directory under `~/.malvin_home/logs/`.

## Summary

| | |
|---|---|
| Agent session | None |
| `.malvin/` | Not required |
| Output | Prefixed model list + `Current:` footer (see below) |

## Intention

Discover valid `--model` values for other malvin commands. Cursor ACP models use the `cursor:` prefix; OpenRouter (malvin-mini) models use the `openrouter:` prefix; local MLX models use the `local:` prefix.

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
4. Print built-in `local:` models with cache status (`cached` / `needs download`).
5. Print blank line and: `Current: <model>` (from `~/.malvin_home/config.toml`, else `cursor:auto`).

`malvin models download local:<id>` fetches weights into `~/.malvin_home/model_cache/` (Apple Silicon / MLX). Known v1 ids: `local:qwen35_9b_q4`, `local:nemotron_cascade2`.

## Examples

```text
malvin models
malvin --no-color models
malvin models download local:qwen35_9b_q4
malvin --model local:qwen35_9b_q4 do "say hi"
malvin --model cursor:sonnet-4 kpop plan.md    # --model applies to agent subcommands, not models
```
