# malvin invent

One **single-turn** MBC2 (boundary exploration) session: structurally distant ideas from your prompt, without evaluation or pruning.

## Summary

| | |
|---|---|
| Input | `<REQUEST>` text or `@file` |
| Prompt | `default_prompts/mbc2.md` with `num_ideas` and `user_prompt` |
| Log | `ideas.log` |

## Intention

Batch creative exploration separate from the `kpop` hypothesis loop. Use before committing to `code` or a long investigation.

## Usage

```text
malvin invent [OPTIONS] <REQUEST>
```

## Arguments

### `<REQUEST>` (required)

Topic as literal text, or `@<path>` (same rules as `do` / `kpop`).

## Options

### `--num-ideas <N>` (default: 3)

Ideas to request in the rendered prompt (`{{ num_ideas }}` in `mbc2.md`).

## Global options

See `malvin --doc`. `--no-markdown` styles agent stdout when enabled.

## Prompt workflow

Exactly **one** coder prompt: rendered `mbc2.md` only (no coding header or repo rules merge).

## Session behavior

- Backs up `.kissconfig`, `.kissignore`, `.malvin/checks`; restores after.
- Checks `result.md` for `ABORT:` after the session.

## Related commands

| Command | When |
|---------|------|
| `malvin kpop` | Test hypotheses and record `## Step` lines |
| `malvin do` | General single-turn coding task with full header |

## Examples

```text
malvin invent "Alternative cache invalidation strategies for our API"
malvin invent --num-ideas 5 @notes/topic.md
```
