# malvin code

Implement a **plan** using malvin’s **KPop gate loop**: repeated agent sessions scoped by `code_constraints.md` until the mpc plan file declares `DONE` and quality gates pass.

## Summary

| | |
|---|---|
| Input | One or more plans (text or `.md` path) → `plan.md` per run dir |
| Loop | Outer gate iterations; each runs one KPop session |
| Success | mpc plan file (`_kpop/mpc_plan.md`) contains exactly `DONE` **and** passing quality gates from `.malvin/checks` (git repo root when inside a git work tree, else `~/.malvin/checks`) |
| Requires | `kiss` on PATH; Cursor agent CLI |

## Intention

Take a written plan and drive the workspace to a gate-clean state while following coding rules embedded in prompts. This is the primary “build this feature” command.

## Usage

```text
malvin code [OPTIONS] <PLAN>...
```

## Arguments

### `<PLAN>...` (required, one or more)

Exactly **one shell argument**. Quote for internal spaces (e.g. `malvin code "Add widget API per plan.md"`). Plan text or a path to an existing `.md` file (no whitespace in the path; case-sensitive `.md` suffix). Copy stored as `plan.md` in the run directory. Nonexistent `.md` paths are treated as literal text.

When multiple plans are given, malvin runs `malvin code` on each in sequence. Each plan gets its own run directory under `~/.malvin_home/logs/<hash>/`, equivalent to separate shell invocations.

## Options

### `--max-loops <N>` (default: 3)

Outer gate-loop budget. Malvin runs up to `max(N, 1) + 1` outer iterations (see `malvin --doc`, section “Gate-loop commands”).

### `--tenacious` (default: on)

Sets `--max-acp-retries=9999` and `--max-loops=9999`.

### `--no-tenacious`

Restore normal loop/retry budgets (global flag; see `malvin --doc`).

## Global options

See `malvin --doc`: `--model`, `--no-force`, `--no-tee`, `--no-markdown`, `--verbose`, `--no-color`, `--background`, `--max-acp-retries`, `--doc`.

## Workflow

1. **Startup** — Create run dir, copy plan to `plan.md`, emit command line and paths.
2. **Gate loop** (`KPopHardConstraints::CODE`) — Unlike `tidy`, **always** enters the loop (no “gates already pass” fast path).
3. **Per outer iteration:**
   - Render `kpop_program.md` with `code_constraints.md` as scope into `plan.md`.
   - Clear `_kpop/mpc_plan.md` at iteration start (malvin resets the scratch plan between outer iterations).
   - Run one KPop agent session (`header.md` + `kpop_common.md` + `mpc_block.md`); log to `kpop.log` and `_kpop/exp_log_<iteration>.md`.
   - Snapshot at each outer iteration; restore after each prompt: `.kissconfig`, `.kissignore`, `.gitignore`, `.malvin/checks`, `.malvin/config.toml`, and `~/.malvin_home/config.toml` (global defaults).
   - Restore all protected files immediately before post-session quality gates (gate pass/fail is not proof of restore).
   - Track whether the mpc plan file contains exactly `DONE`.
4. **Exit** — Success when mpc plan `DONE` aligns with passing workspace gates; otherwise fail after exhaustion (gates rechecked).

## Prompt roles

| Artifact | Role |
|----------|------|
| `code_constraints.md` | Plan-specific scope (constraints, plan path) |
| `kpop_program.md` | Rendered into `plan.md` — scope constraints + quality gates |
| `kpop_common.md` | Popper method: hypothesize → predict → falsify; log to experiment log |
| `mpc_block.md` | MPC workflow per session: plan → review → revise → implement → `DONE` in mpc plan file |
| `header.md` | Prepended on each gate-loop agent turn |

## Artifacts

- `~/.malvin_home/logs/<hash>/<run>/plan.md` — input plan
- `_kpop/mpc_plan.md` — per-iteration MPC plan scratch file
- `_kpop/exp_log_*.md` — experiment log (hypotheses and test results)
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
malvin code plan_1.md plan_2.md plan_3.md
malvin code --max-loops 3 "Add widget API per plan.md"
malvin --model sonnet-4 code plans/feature.md
```
