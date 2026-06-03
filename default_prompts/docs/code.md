# malvin code

Implement a **plan** using malvin’s **KPop gate loop**: repeated agent sessions scoped by `code_constraints.md` until quality gates pass and the experiment log records consecutive success.

## Summary

| | |
|---|---|
| Input | Plan text or path to `.md` → `plan.md` in the run dir |
| Loop | Outer gate iterations; each runs one KPop session |
| Success | Two consecutive `## KPOP_SOLVED` markers **and** passing `.malvin/checks` gates |
| Requires | `kiss` on PATH; Cursor agent CLI |

## Intention

Take a written plan and drive the workspace to a gate-clean state while following coding rules embedded in prompts. This is the primary “build this feature” command.

## Usage

```text
malvin code [OPTIONS] <REQUEST>
```

## Arguments

### `<REQUEST>` (required)

Plan text or a path to an existing `.md` file (no whitespace in the path; case-sensitive `.md` suffix). Copy stored as `plan.md` in the run directory. Nonexistent `.md` paths are treated as literal text.

## Options

### `--max-loops <N>` (default: 3)

Outer gate-loop budget. Malvin runs up to `max(N, 1) + 1` outer iterations (see `malvin --doc`, section “Gate-loop commands”).

### `--max-hypotheses <N>` (default: 10)

Per-session hypothesis budget: maximum `## Step … — KPOP` lines the agent should add in one gate-loop iteration (`{{ want }}` in the rendered prompt).

### `--tenacious` (default: on)

Sets `--max-acp-retries=9999` and `--max-loops=9999`.

### `--no-tenacious`

Restore normal loop/retry budgets (global flag; see `malvin --doc`).

## Global options

See `malvin --doc`: `--model`, `--no-force`, `--no-tee`, `--no-markdown`, `--verbose`, `--no-color`, `--background`, `--max-acp-retries`, `--doc`.

## Workflow

1. **Startup** — Create run dir, copy plan to `plan.md`, emit command line and paths.
2. **Gate loop** (`GateLoopBehavior::CODE`) — Unlike `tidy`, **always** enters the loop (no “gates already pass” fast path).
3. **Per outer iteration:**
   - Render `kpop_program.md` with `code_constraints.md` as scope.
   - Run one KPop agent session; log to `kpop.log` and `_kpop/exp_log_<iteration>.md`.
   - Snapshot/restore `.kissconfig`, `.kissignore`, `.malvin/checks`.
   - Track consecutive sessions that end with `## KPOP_SOLVED`.
4. **Exit** — Success when two consecutive solved markers align with passing workspace gates; otherwise fail after exhaustion (gates rechecked).

## Prompt roles

| Artifact | Role |
|----------|------|
| `code_constraints.md` | Plan-specific scope (constraints, plan path) |
| `kpop_program.md` | Shared KPop multiturn instructions + quality gates |
| `header.md` / coding rules | Prepended on first turn via shared KPop machinery |

## Artifacts

- `./.malvin/logs/<run>/plan.md` — input plan
- `_kpop/exp_log_*.md` — experiment log (authoritative for KPop steps)
- `kpop.log` — session transcript
- `quality_gates.log` — gate command output
- `result.md` — `ABORT:` stops the workflow when checked

## Related commands

| Command | When |
|---------|------|
| `malvin tidy` | Fix gates without a feature plan |
| `malvin kpop` / bare `malvin` | Investigation without a shipping plan |

## Examples

```text
malvin code plan.md
malvin code --max-loops 3 --max-hypotheses 15 "Add widget API per plan.md"
malvin --model sonnet-4 code @plans/feature.md
```
