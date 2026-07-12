# malvin models

List model ids for malvin runs. No malvin prompts and no run directory under `~/.malvin_home/logs/`.

## Summary

| | |
|---|---|
| Agent session | None |
| `.malvin/` | Not required |
| Output | Prefixed model list + `Current:` footer (see below) |

## Intention

Discover valid `--model` values for other malvin commands. Cursor ACP models use the `cursor:` prefix; OpenRouter (malvin-mini) models use the `openrouter:` prefix.

## Usage

```text
malvin models [OPTIONS]
```

## Global options

See `malvin --doc`. Only `--no-color` materially affects output formatting. Global `--model` is parsed but **not used** by this subcommand.

## Behavior

1. Resolve `agent` or `cursor-agent` on `PATH`.
2. Run `<binary> models` and print each id with a `cursor:` prefix.
3. Fetch OpenRouter models (when available) and print each id with an `openrouter:` prefix.
4. Print blank line and: `Current: <model>` (from `~/.malvin_home/config.toml`, else `cursor:auto`).

## Examples

```text
malvin models
malvin --no-color models
malvin --model cursor:sonnet-4 kpop plan.md    # --model applies to agent subcommands, not models
```
