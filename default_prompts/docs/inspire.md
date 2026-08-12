# malvin inspire

One **single-turn** MBC2 (boundary exploration) session: structurally distant ideas from your prompt, without evaluation or pruning.

## Summary

| | |
|---|---|
| Input | `<REQUEST>` text or existing `.md` path |
| Prompt | `default_prompts/mbc2.md` with `user_prompt` |
| Log | `inspire.log` under `~/.malvin_home/logs/<hash>/<run>/` |

## Intention

Batch creative exploration separate from the default `router_a` / `router_b` route. Use before committing to a long investigation or implementation run.

## Usage

```text
malvin inspire [OPTIONS] [REQUEST]
```

If `REQUEST` is omitted (and `--doc` is not set), malvin prints short usage on stdout and exits 0.

## Arguments

### `[REQUEST]`

Required to run. Exactly **one shell argument**. Quote for internal spaces. Topic as literal text, or an existing `.md` file path (same rules as `--do`).

## Global options

See `malvin --doc`. Agent stdout uses styled markdown on a TTY.

## Prompt workflow

Exactly **one** coder prompt: rendered `mbc2.md` only (no coding header or repo rules merge). The prompt instructs the model to generate 3 ideas when no count is specified.

## Session behavior

- Ensures `~/.malvin_home/config.toml` exists with defaults (same as `--do` and `tidy`).
- Backs up `.gitignore`, `.malvin/checks`, and `.malvin/config.toml`; restores after the session.
- Checks `result.md` for `ABORT:` after the session.

## Related commands

| Command | When |
|---------|------|
| `malvin --do` | One-shot agent turn with full header |

## Examples

```text
malvin inspire "Alternative cache invalidation strategies for our API"
malvin inspire notes/topic.md
```
