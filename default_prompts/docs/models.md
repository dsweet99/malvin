# malvin models

List model ids available from the installed Cursor agent CLI. Does not invoke malvin prompts or create a `_malvin` run directory.

## Intention

Discover which `--model` values can be passed to malvin subcommands that spawn an agent. Shows the malvin default model id for comparison.

## Usage

```text
malvin models [OPTIONS]
```

## Options

Only **global** options apply (`malvin.md`):

- `--no-color`
- `--model` — Passed through CLI parsing but **not used** by this subcommand (listing is independent of selection).
- `--no-force`, `--no-tee`, `--no-markdown`, `--verbose` — No effect on `models` (no agent session).

## Behavior

1. Resolve `agent` or `cursor-agent` on `PATH`.
2. Run `<binary> models`.
3. Strip ANSI escapes and trailing “Tip:” banner lines from stdout.
4. Parse model names when output looks like a bullet list; otherwise print cleaned stdout verbatim.
5. Print blank line and: `Default model in malvin: <DEFAULT_CLI_MODEL>` (currently `auto` unless changed in malvin config).

## Requirements

- `agent` or `cursor-agent` on PATH

## Does not require

- `kiss`
- Workspace or `_malvin` directory

## No prompt workflow

This command does not run any malvin prompt templates.

## Example

```text
malvin models
malvin --no-color models
```

Use a listed id with other commands:

```text
malvin --model sonnet-4 code @plan.md
```
