# malvin tidy

Bring the workspace back to a **gate-clean** state by composing a fixed request and running the **default router** workflow (same path as bare `malvin REQUEST`) with **`--gates` forced on**.

## Summary

| | |
|---|---|
| Input | None (fixed request: `Get the gates to pass.`) |
| Loop | Default router: `header` → `kpop_common` → `router_a` → optional `router_b`; exit `router_summarize`; outer `--max-loops` sessions |
| Gates | Always on — workspace `.malvin/checks` are harness loop/exit criteria |
| Fast path | **None** — always runs the router |
| Requires | Agent backend for chosen `--model`; `.malvin/checks` needed for gate pass/fail (use `malvin init` to discover) |

## Intention

Recover after CI drift or local gate failures—without a feature plan. Tidy is a thin wrapper around the default router with a fixed prompt and gates enabled.

## Usage

```text
malvin tidy [OPTIONS]
```

No positional arguments. Work directory is always `.` (cwd).

## Options

### `--max-loops <N>` (default: 3)

Outer router session budget (`effective_max_loops`). `0` is treated as `1`.

### `--tenacious` (default: on)

Sets `--max-acp-retries=9999` and `--max-loops=9999`.

### `--no-tenacious`

Restore normal loop/retry budgets (global flag; see `malvin --doc`).

## Global options

See `malvin --doc`. Tidy always enables harness `--gates`, whether or not you pass `--gates` on the CLI. `--quiet` / `-q` applies because tidy invokes the default router (DM-body-only stdout; not the same as `-b`).

## Workflow

1. Compose the fixed request text `Get the gates to pass.`
2. Force `--gates` on and invoke the default router (same engine as bare `malvin REQUEST`).
3. After each outer session, run workspace `.malvin/checks`: pass stops success; fail continues until the outer budget is exhausted (then fail).

## Artifacts

Same as the default router under `~/.malvin_home/logs/<hash>/<run>/` (for example `plan_*.md`, `quality_gates.log`, `_kpop/`, `stdout.log`).

## Related commands

| Command | When |
|---------|------|
| `malvin init` | Discover and write `.malvin/checks` without running tidy |
| bare `malvin REQUEST --gates` | Same router engine; tidy is a thin fixed-request wrapper |

## Examples

```text
malvin tidy
malvin tidy --max-loops 5
```
