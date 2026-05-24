# malvin plan (experimental)

Write or **review** a plan file using a single agent session. Does not implement the plan—that is `malvin code`.

## Intention

Produce or improve `plan.md` (or another path) with malvin’s coding header, merged repo rules, and review-plan instructions—without entering the implement/review loop.

## Usage

```text
malvin plan [OPTIONS] [TEXT]
```

## Arguments

### `[TEXT]` (optional)

How the plan destination and content are determined:

| Form | Destination | Content |
|------|-------------|---------|
| (omit) | `./plan.md` | Review existing file (must exist) |
| Plain text | `./plan.md` | Write normalized text to `plan.md` |
| `path.md` only (no `--plan_path`) | `path.md` in place | Review existing file; session work dir = file’s parent |
| `src.md` + `--plan_path <dest>` | `<dest>` | Copy/read from `src.md` into destination |
| Text + `--plan_path` | Flag path | Write text to flag path |

Normalization: trim trailing newlines, ensure single trailing newline; empty-after-trim text errors.

A positional string is treated as a file path only when it has no whitespace, ends with `.md` (case-sensitive), has no invalid path characters, and names an existing regular file. Otherwise it is literal plan text (including nonexistent `.md` paths).

**In-place file review:** When positional is only an existing `.md` file and no `--plan_path`, malvin does not rewrite the source file; it opens a session in the file’s directory and reviews that path.

## Options

### `--plan_path <PATH>` / `--plan-path <PATH>`

Explicit plan file path (default when positional is not an existing `.md` file: `plan.md` in cwd). Relative paths resolve from cwd.

### Global options

See `malvin.md`. Uses standard coding-session stdout markdown unless `--no-markdown`.

## Requirements

- `kiss` on PATH
- Cursor agent CLI

## Prompt workflow

**One** coder prompt per invocation (no implement/review loop).

| Step | Prompt role (effect) |
|------|----------------------|
| 1 | **Coding header** — Malvin identity, history/memory, paths. |
| 2 | **Coding rules** — Default rules plus repo overrides from `.malvin_memory` / workspace. |
| 3 | **Review plan** — Agent writes or critiques the plan at `plan_path`; structured review expectations (LGTM / feedback in `review.md` per template). |

Logged as `review_plan` → `review_plan.log`.

No `implement`, `concerns`, `learn`, or post-run gate loop in this command.

## Session behavior

- Work directory: parent of plan file, or directory from existing `.md` resolution rules above.
- Run dir: `_malvin/<stamp>/` under session work dir.
- Dotfile backup/restore like other coding sessions.
- Abort via `result.md` honored after the prompt.

## Relationship to `code`

| Step | `plan` | `code` |
|------|--------|--------|
| Plan authoring/review | Yes | Uses existing plan |
| `check_plan` | No (review embedded in review_plan) | Yes (unless `--trust-the-plan`) |
| Implement | No | Yes |

Typical flow:

```text
malvin plan "Add caching layer for API responses"
malvin code --trust-the-plan plan.md
```

## Examples

```text
malvin plan
malvin plan "Refactor auth module"
malvin plan draft.md --plan_path plan.md
malvin plan plan.md
```
