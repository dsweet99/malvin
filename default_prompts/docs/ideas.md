# malvin ideas

Run a **single** MBC2 (boundary exploration) ideation turn: the agent generates structurally distant ideas from your prompt without evaluating or pruning them.

## Intention

Batch creative exploration separate from the full `kpop` hypothesis loop. Uses `default_prompts/mbc2.md` with your request and a configurable idea count.

## Usage

```text
malvin ideas [OPTIONS] <REQUEST>
```

## Arguments

### `<REQUEST>` (required)

The user’s topic as literal text, or `@<path>` to read from a file (same rules as `do` / `kpop`).

## Options

### `--num-ideas <N>`

Number of ideas to ask for in the rendered prompt. Default: **3**. Maps to `{{ num_ideas }}` in `mbc2.md`.

### Global options

See `malvin.md`. Agent stdout is **plain/raw** (like `do`); `--no-markdown` does not change styling. `--verbose` logs full prompt bodies.

## Prompt workflow

Exactly **one** coder prompt per invocation: rendered `mbc2.md` only (no coding header or repo rules merge).

| Step | Role | Log |
|------|------|-----|
| 1 | **MBC2 prompt** — boundary-exploration instructions plus `num_ideas` and `user_prompt` | `ideas.log` |

## Session behavior

- Backs up `.kissconfig`, `.kissignore`, `.malvin_checks` before the session; restores after.
- Writes `ideas.log`, `stdout.log`, `prompts.log`, timing JSON as applicable.
- Checks `result.md` for ABORT after the session (same abort protocol as `do`).

## When to use

- Explore many idea variants before committing to a `kpop` or `code` workflow.
- One-shot brainstorming piped to a file or terminal.

## Examples

```text
malvin ideas "Alternative cache invalidation strategies for our API"
malvin ideas --num-ideas 5 @notes/topic.md
```
