# malvin priors

Ground a user request in **good priors** via the KPop gate loop scoped by `priors_constraints.md`. The agent writes a markdown priors report to a workspace path you choose.

## Summary

| | |
|---|---|
| Input | Required request text or `.md` path |
| Output | Workspace file at `--out-path` (default: `priors.md`) |
| Loop | Full gate-kpop loop (`KPopHardConstraints::PRIORS`) |
| Fast path | **None** — always runs the agent (unlike `tidy`) |
| Exit policy | Valid non-empty output at `--out-path`; workspace gates need not pass |
| Requires | No `.malvin/checks` preflight at CLI entry (document workflow, like `delight` / `explain` / `revise`) |

## Intention

Reduce epistemic uncertainty in a user request by researching conventions, best practices, and near-neighbor work, then reporting useful facts and references tied to those uncertainties. Typical pipeline: `malvin priors REQUEST` → read the report → bare `malvin REQUEST` with clearer decisions.

## Usage

```text
malvin priors [REQUEST] [OPTIONS]
```

### `[REQUEST]` (required)

Literal text or path to an existing `.md` file. Malvin writes the resolved request to the run's `user_request.md` and injects that path into `priors_constraints.md`.

## Options

### `--out-path <PATH>` (default: `priors.md`)

Workspace path for the generated priors report. With the default `priors.md`, if that file already exists, malvin allocates the first free sibling (`priors_1.md`, `priors_2.md`, …) before the agent runs. For any other `--out-path`, if the path already exists (regular file, empty file, directory, or symlink to an existing target), the command exits immediately with:

```text
malvin priors: `<path>` already exists; refusing to overwrite
```

No run artifacts or agent work starts when a non-default path pre-exists.

### `--max-loops <N>` (default: 3)

Outer gate-loop budget (`max(N, 1) + 1` iterations). `0` is treated as `1`.

### `--tenacious` (default: on)

Sets `--max-acp-retries=9999` and `--max-loops=9999`.

### `--no-tenacious`

Restore normal loop/retry budgets (global flag; see `malvin --doc`).

## Global options

See `malvin --doc`.

## Success criteria

All of the following must hold:

1. Preflight passed (default `priors.md` may have been auto-allocated to a sibling; non-default paths must not have pre-existed).
2. Agent completed within the `--max-loops` budget.
3. After the session, `--out-path` is a regular file with size &gt; 0.

On success, malvin prints `DONE` to stdout.

## Related commands

| Command | When |
|---------|------|
| `malvin delight` | Author a delight pitch (same gate-loop style) |
| `malvin kpop` | Open-ended scientific investigation |

## Examples

```text
malvin priors "Add a retry budget flag"
malvin priors plans/feature.md
malvin priors plans/feature.md --out-path plans/feature_priors.md
malvin "Implement the plan in plans/feature.md using plans/feature_priors.md"
```
