# malvin delight

Author a **user-delighting feature plan** via the KPop gate loop scoped by `delight_constraints.md`. The agent writes a new markdown plan to a workspace path you choose.

## Summary

| | |
|---|---|
| Input | None |
| Output | Workspace file at `--out-path` (default: `plan.md`) |
| Loop | Full gate-kpop loop (`GateLoopBehavior::DELIGHT`) |
| Fast path | **None** — always runs the agent (like `code`, unlike `tidy`) |
| Exit policy | One `## KPOP_SOLVED` in the session exp log; workspace gates need not pass |
| Requires | `kiss` on PATH (same preflight as `code` / `tidy`) |

## Intention

Generate a fresh, repo-grounded plan for a feature or improvement that would delight the user — without overwriting an existing plan file. On success, malvin automatically runs `malvin plan` on the same `--out-path`. Typical pipeline: `malvin delight` → `malvin code <out-path>`.

## Usage

```text
malvin delight [OPTIONS]
```

No positional arguments.

## Options

### `--out-path <PATH>` (default: `plan.md`)

Workspace path for the generated plan. **Fail-if-exists:** if this path already exists (regular file, empty file, directory, or symlink to an existing target), the command exits immediately with:

```text
malvin delight: `<path>` already exists; refusing to overwrite
```

No run artifacts or agent work starts when the path pre-exists.

### `--max-loops <N>` (default: 3)

Outer gate-loop budget (`max(N, 1) + 1` iterations). `0` is treated as `1`.

### `--max-hypotheses <N>` (default: 5)

Hypothesis budget per KPop session inside the gate loop.

### `--tenacious` (default: on)

Sets `--max-acp-retries=9999` and `--max-loops=9999`.

### `--no-tenacious`

Restore normal loop/retry budgets (global flag; see `malvin --doc`).

## Global options

See `malvin --doc`.

## Success criteria

All of the following must hold:

1. Preflight passed (`--out-path` did not exist at start).
2. The agent declared `## KPOP_SOLVED` in a session exp log.
3. After the session, `--out-path` is a regular file with size &gt; 0.
4. The decoupled `malvin plan` workflow runs automatically on the same `--out-path` (overwrites it with the revised implementation plan from Prompt 3).

The output file does **not** need plan-pipeline section headings when the delight session finishes; the chained `malvin plan` step produces the final normative spec (no user prefix, no `BEGIN_MALVIN` block).

On success, malvin prints `DONE` to stdout.

## Related commands

| Command | When |
|---------|------|
| `malvin inspire` | One-shot MBC2 ideation; no plan file |
| `malvin plan` | Four-prompt refinement on an existing plan (runs automatically after `delight`; also available standalone) |
| `malvin code` | Implement a plan via the gate loop |

## Examples

```text
malvin delight
malvin delight --out-path plans/feature.md
malvin code plans/feature.md
```
