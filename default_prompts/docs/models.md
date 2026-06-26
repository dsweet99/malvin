# malvin models

List model ids from the installed Cursor agent CLI. No malvin prompts and no `./.malvin/logs` run directory.

## Summary

| | |
|---|---|
| Agent session | None |
| kiss / `.malvin/` | Not required |
| Output | Cleaned `agent models` list + malvin default model line |

## Intention

Discover valid `--model` values for other malvin commands (ACP / Cursor CLI). For `--mini`, model slugs come from OpenRouter; configure `[agent]."model-mini"` or pass `--model` (see `malvin --doc` under `--mini`).

## Usage

```text
malvin models [OPTIONS]
```

## Global options

See `malvin --doc`. Only `--no-color` materially affects output formatting. `--model` is parsed but **not used** by this subcommand.

## Behavior

1. Resolve `agent` or `cursor-agent` on `PATH`.
2. Run `<binary> models`.
3. Strip ANSI escapes and trailing “Tip:” banner lines.
4. Parse bullet-list model names when possible; otherwise print cleaned stdout verbatim.
5. Print blank line and: `Default model in malvin: <DEFAULT_CLI_MODEL>` (currently `auto`).

## Examples

```text
malvin models
malvin --no-color models
malvin --model sonnet-4 code plan.md    # --model applies to code, not models
```
