# malvin init

Discover how the workspace runs quality gates today and write `.malvin/gates` (one shell command per non-empty line).

## Summary

| | |
|---|---|
| Input | None (fixed request from `init_constraints.md`) |
| Loop | Default router: `header` → `router_a` → optional `router_b`; exit `router_summarize`; outer `--max-loops` sessions |
| Gates | Off by default — init asks the agent to create `.malvin/gates`; harness gates are not forced |
| Fast path | **None** — always runs the router |
| Requires | Agent backend for chosen `--model` |

## Intention

Bootstrap a repo for gated workflows (`malvin tidy`, bare `malvin REQUEST --gates`) without running those workflows. Use this when you want `.malvin/gates` materialized explicitly.

## Usage

```text
malvin init [OPTION]...
```

No positional arguments. Work directory is always `.` (cwd).

## Behavior

1. Render `init_constraints.md` with the absolute cwd as `repo_root_path`.
2. Invoke the default router with that request (same engine as bare `malvin REQUEST`).
3. Normal router success/stop behavior applies; malvin does not skip when `.malvin/gates` already exists and does not post-check for the file.

Delete `.malvin/gates` and run `malvin init` again if you want the agent to rediscover gates.

## Options

### `--max-loops <N>` (default: 3)

Outer router session budget. `0` is treated as `1`.

### `--max-hypotheses <N>` (default: 5)

Hypothesis budget for the router session.

### `--tenacious` (default: on)

Sets `--max-acp-retries=9999` and `--max-loops=9999`.

### `--no-tenacious`

Restore normal loop/retry budgets (global flag; see `malvin --doc`).

## Global options

See `malvin --doc`. Init does **not** force `--gates`. `--quiet` / `-q` applies because init invokes the default router (DM-body-only stdout; not the same as `-b`).

## Notes

- Discovery uses repo signals only; malvin does not invent default linters or test runners when the repo provides no signal.
- Comment lines in `.malvin/gates` start with `#` after trimming and are ignored when running gates.
