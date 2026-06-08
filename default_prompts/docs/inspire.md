# malvin inspire

One **single-turn** MBC2 (boundary exploration) session: structurally distant ideas from your prompt, without evaluation or pruning.

## Summary

| | |
|---|---|
| Input | `<REQUEST>` text or `@file` |
| Prompt | `default_prompts/mbc2.md` with `user_prompt` |
| Log | `ideas.log` |

## Intention

Batch creative exploration separate from the `kpop` hypothesis loop. Use before committing to `code` or a long investigation.

## Usage

```text
malvin inspire [OPTIONS] <REQUEST>
```

## Arguments

### `<REQUEST>` (required)

Topic as literal text, or `@<path>` (same rules as `do` / `kpop`).

## Global options

See `malvin --doc`. `--no-markdown` styles agent stdout when enabled.

## Prompt workflow

Exactly **one** coder prompt: rendered `mbc2.md` only (no coding header or repo rules merge). The prompt instructs the model to generate 3 ideas when no count is specified.

## Session behavior

- Backs up `.kissconfig`, `.kissignore`, `.gitignore`, `.malvin/checks`, `.malvin/config.toml`; restores after the session.
- Checks `result.md` for `ABORT:` after the session.

## Related commands

| Command | When |
|---------|------|
| `malvin kpop` | Test hypotheses and record `## Step` lines |
| `malvin do` | General single-turn coding task with full header |

## Examples

```text
malvin inspire "Alternative cache invalidation strategies for our API"
malvin inspire @notes/topic.md
```
