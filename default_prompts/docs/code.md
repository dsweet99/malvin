# malvin code

Implement a **plan** using the full malvin coding pipeline: optional plan validation, implementation, multi-attempt code review, optional learning, and a closing summary.

## Intention

Take a written plan (inline or from a file) and drive the repo to a reviewed state with quality gates enforced before each review cycle. This is the primary “build this feature” command.

## Usage

```text
malvin code [OPTIONS] <REQUEST>
```

## Arguments

### `<REQUEST>` (required)

Plan text or a path to an existing `.md` file (no whitespace in the path string; case-sensitive `.md` suffix). Copy stored as `plan.md` in the run directory. Nonexistent `.md` paths are treated as literal text.

## Options

### `--max-loops <N>` (default: 5)

Maximum **review cycles** after implementation. Each cycle:

1. Reviewer fan-out + review write aggregation
2. If not LGTM: **concerns** coder turn to address feedback
3. Repeat until LGTM or budget exhausted

Value `0` is treated as `1` (at least one review attempt). Plan check retries also respect this budget.

### `--no-learn`

Skip the **learn** prompt after a successful review (even if elapsed time would normally allow learn).

### `--trust-the-plan`

Skip the **check plan** phase. Use when the plan was already reviewed (e.g. via `malvin plan`) and you want to go straight to implementation.

### `--skip-pre-checks`

Skip workspace quality gates **before** the ACP session starts. Default: gates must pass or malvin exits with guidance to run `malvin tidy` or retry with this flag.

### Global options

See `malvin.md`. `--no-markdown` affects agent stdout styling. `--no-force` disables agent `--force`. Learn runs unless `--no-learn` is set on the subcommand (there is no global `--no-learn`).

## Requirements

- `kiss` on PATH (CLI refuses to start otherwise)
- Cursor agent CLI
- Passing pre-checks unless `--skip-pre-checks`

## Prompt workflow

Single long-lived coder session: main work, then optional learn, then summary.

### Phase A — Before summary (main loop)

| Order | Prompt role (effect) | Notes |
|-------|----------------------|-------|
| 1 | **Check plan** (skipped if `--trust-the-plan`) — Agent reviews `plan.md`; must write LGTM (or actionable feedback) to `review.md`. Failure aborts the run. | Retries if review file missing |
| 2 | **Implement** — Agent implements the plan in the workspace. | Main coding turn |
| 3–N | **Review loop** (up to `--max-loops`) | See below |

**Each review iteration:**

| Sub-step | Prompt role (effect) |
|----------|----------------------|
| 3a | **Pre-review quality gates** — Run `.malvin/checks` commands. If any fail, malvin writes gate output to `review.md` and skips to concerns (3d). |
| 3b | **Reviewers spawn** — Parallel reviewer agents produce structured review material into run artifacts. |
| 3c | **Review write** — Aggregates reviewer output into a single review verdict in `review.md`. |
| 3d | If not LGTM: **Concerns** — Agent addresses review feedback and updates code. |
| 3e | Abort check on `result.md` between steps. |

Loop exits on LGTM or exhausted `--max-loops`.

| Order | Prompt role (effect) | Notes |
|-------|----------------------|-------|
| 4 | **Learn** (optional) — Reflect on the session and suggest process/repo improvements. Skipped if `--no-learn`, or if elapsed time &lt; 5 minutes (default threshold). | After review succeeds |
| 5 | **Summary** — Short user-facing recap of what was done. |  |

## Artifacts

- `./.malvin/logs/<run>/plan.md` — input plan
- `review.md`, `review_prep.md` — review pipeline
- `result.md` — `ABORT:` prefix stops the workflow
- `quality_gates.log` — gate commands
- Phase logs: `check_plan`, `implement`, `concerns`, `learn`, `summary`, etc.

## Session safety

`.kissconfig`, `.kissignore`, and `.malvin/checks` are snapshotted before the session and restored after so agent edits do not persist unintentionally.

## Examples

```text
malvin code plan.md
malvin code --trust-the-plan plan.md --max-loops 3
malvin code --skip-pre-checks "Add widget API per plan.md"
```
