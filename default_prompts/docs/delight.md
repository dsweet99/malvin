# malvin delight

Author a **user-delighting feature pitch** via the KPop gate loop scoped by `delight_constraints.md`. The agent writes a new markdown pitch to a workspace path you choose.

## Summary

| | |
|---|---|
| Input | Optional guidance text or `.md` path |
| Output | Workspace file at `--out-path` (default: `pitch.md`) |
| Loop | Full gate-kpop loop (`KPopHardConstraints::DELIGHT`) |
| Fast path | **None** — always runs the agent (like `code`, unlike `tidy`) |
| Exit policy | Two consecutive `## KPOP_SOLVED` markers in per-iteration exp logs; workspace gates need not pass |
| Requires | No `kiss` or `.malvin/checks` preflight at CLI entry (document workflow, like `explain` / `revise`) |

## Intention

Generate a fresh, repo-grounded pitch for a feature or improvement that would delight the user — without overwriting an existing pitch file. Typical pipeline: `malvin delight` → `malvin code <out-path>`.

## Usage

```text
malvin delight [GUIDANCE] [OPTIONS]
```

### `[GUIDANCE]` (optional)

Literal text or path to an existing `.md` file. When provided, malvin injects the resolved text into `delight_constraints.md` so the agent steers the pitch toward your guidance. Omitted guidance preserves the current behavior.

## Options

### `--out-path <PATH>` (default: `pitch.md`)

Workspace path for the generated pitch. With the default `pitch.md`, if that file already exists, malvin allocates the first free sibling (`pitch_1.md`, `pitch_2.md`, …) before the agent runs. For any other `--out-path`, if the path already exists (regular file, empty file, directory, or symlink to an existing target), the command exits immediately with:

```text
malvin delight: `<path>` already exists; refusing to overwrite
```

No run artifacts or agent work starts when a non-default path pre-exists.

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

1. Preflight passed (default `pitch.md` may have been auto-allocated to a sibling; non-default paths must not have pre-existed).
2. Two consecutive outer gate-loop iterations each declared `## KPOP_SOLVED` in their own exp log.
3. After the session, `--out-path` is a regular file with size &gt; 0.

On success, malvin prints `DONE` to stdout.

## Related commands

| Command | When |
|---------|------|
| `malvin inspire` | One-shot MBC2 ideation; no pitch file |
| `malvin code` | Implement a plan via the gate loop |

## Examples

```text
malvin delight
malvin delight "Improve error messages for gate failures"
malvin delight guidance.md
malvin delight --out-path plans/feature.md
malvin code plans/feature.md
```
