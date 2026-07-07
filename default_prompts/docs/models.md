# malvin models

List model ids for malvin runs. No malvin prompts and no run directory under `~/.malvin_home/logs/`.

## Summary

| | |
|---|---|
| Agent session | None |
| `.malvin/` | Not required |
| Output | Model list + default model footer (see below) |

## Intention

Discover valid `--model` values for other malvin commands via the Cursor agent CLI.

## Usage

```text
malvin models [OPTIONS]
```

## Global options

See `malvin --doc`. Only `--no-color` materially affects output formatting. Global `--model` is parsed but **not used** by this subcommand.

## Behavior

1. Resolve `agent` or `cursor-agent` on `PATH`.
2. Run `<binary> models`.
3. Strip ANSI escapes and trailing “Tip:” banner lines.
4. Parse bullet-list model names when possible; otherwise print cleaned stdout verbatim.
5. Print blank line and: `Default model: auto`.

## Examples

```text
malvin models
malvin --no-color models
malvin --model sonnet-4 kpop plan.md    # --model applies to agent subcommands, not models
```
