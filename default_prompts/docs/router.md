# malvin (default route)

Outer agent sessions (`--max-loops`): each session sends `header.md`, then `router_a.md`. A lone-line `__MALVIN_DONE__` in the `router_a` reply can stop the loop (optionally after `--gates` checks). Otherwise the same session receives `router_b.md` (or `router_b_creative.md` with `--creative`), and another outer session may start when budget remains. When exiting, `router_summarize.md` runs once on the final open session.

## Summary

| | |
|---|---|
| Input | `<REQUEST>` text or existing `.md` path |
| Output | Styled stdout on a TTY (same startup chrome as `tidy` / `inspire`); with `--quiet` / `-q`, only `MALVIN_DM_*` bodies |
| Logs | `router_N.log` under `~/.malvin_home/logs/<hash>/<run>/` (one file per outer session) |
| Requires | No `.malvin/checks` at startup (unless `--gates` later needs them) |

## Intention

Read the user request (on disk as `plan_*.md` / `{{ user_request_path }}`), ask whether requirements are still unsatisfied (`router_a.md`), and either stop on `__MALVIN_DONE__` or continue with `router_b.md` (or `router_b_creative.md` with `--creative`) to satisfy them. When the outer loop decides to exit, send `router_summarize.md` once on that final already-open coder session before teardown. Repeat for another outer agent lifetime when `--max-loops` allows and stop conditions are not met.

## Usage

```text
malvin [OPTION]... [REQUEST]
```

There is no `router` subcommand. Bare `malvin REQUEST` is the default autonomous routing workflow. If `REQUEST` is omitted (and no subcommand is given), malvin prints the command catalog on stdout and exits 0.

## Arguments

### `[REQUEST]`

Required to run the default route. Exactly **one shell argument**. Quote for internal spaces. Literal text, or an existing `.md` file path (same rules as `--do`).

| Form | Work directory | Stored as |
|------|----------------|-----------|
| Literal | `.` (cwd) | `plan_<random>.md` in run dir |
| `path/to/file.md` | Parent of file | `plan_<random>.md` |

## Global options

See `malvin --doc`. Notable for the default route:

| Flag | Effect |
|------|--------|
| `--max-loops` | Outer agent-session budget (default 1). Tenacious expands to 9999 unless this flag is set on the command line. |
| `--max-hypotheses` | Hypothesis budget (default 5). When omitted, `[default_workflow].max_hypotheses` is used. Explicit CLI wins over config. |
| `-g` / `--gates` | When `router_a` emits `__MALVIN_DONE__`, run workspace `.malvin/checks`. Pass stops success; fail continues (new outer session). Exhausted budget with failing gates fails the run after exit summarize. Also injects check text into `router_a.md` via `{{ code_extra }}`. |
| `--creative` | Use the creative router_b prompt for the optional work turn |
| `--no-tenacious` | Keep normal `--max-loops` / `--max-acp-retries` (default tenacious expands both) |
| `--quiet` / `-q` | Stdout shows only `MALVIN_DM_*` bodies (not `-b`). Plain `--do` is already DM-body-only without `--verbose` |
| `--verbose` | Full prompt bodies in `prompts.log`; with `--do`, also same live agent stdout log classes as the default workflow |

## Prompt workflow

Each outer session opens one coder session and sends:

| Turn | Piece | Role |
|------|-------|------|
| 1 | `header.md` | Standard Malvin context |
| 2 | `kpop_common.md` | Karl Popper hypothesis-and-falsification method |
| 3 | `router_a.md` | Ask whether requirements are unsatisfied; optional `{{ code_extra }}` when `--gates` |
| 4 (optional) | `router_b.md` or `router_b_creative.md` | Run only when `router_a` did **not** emit `__MALVIN_DONE__` alone on a line; `--creative` selects `router_b_creative.md` |
| Exit only | `router_summarize.md` | **Once per run**, when exiting the outer loop: pass to the same already-open final coder session before teardown |

### Stop / continue (without `--gates`)

After `router_a`, if any line trims to exactly `__MALVIN_DONE__`, skip `router_b` and stop success. Otherwise send `router_b` and, if outer budget remains, tear down **without** summarize and start another session. Exhausting the budget without `--gates` is success (with the single exit summarize on that final session).

### Stop / continue (with `--gates`)

Gates run **only** when `__MALVIN_DONE__` was seen:

| Condition | Action |
|-----------|--------|
| Done + gates pass | Send exit summarize on the open session, tear down, stop success |
| Done + gates fail, loops remain | Tear down **without** summarize; restart |
| Done + gates fail, budget exhausted | Send exit summarize on the open session, tear down, fail with a workspace gate error |
| Not done | Send `router_b`; continue or exit on budget as without gates (gates not run) |

### Required template keys

| Key | Required by | Value source |
|-----|-------------|--------------|
| `user_request_path` | `router_a.md` | run artifacts |
| `code_extra` | `router_a.md` | `router_code_extra.md` when `--gates` |

When the outer loop decides to exit, malvin sends `router_summarize.md` on the same final coder session, then ends the session. Intermediate sessions that continue do not receive summarize.

## Config

`~/.malvin_home/config.toml`:

```toml
[default_workflow]
max_hypotheses = 5
```

Missing section falls back to 5. Explicit `--max-hypotheses` wins over this section. This path does **not** use `[agent].max_hypotheses`.

## Examples

```text
malvin "Investigate flaky tests"
malvin plan.md
malvin --gates "Get the gates to pass"
malvin --creative --max-loops 3 notes/idea.md
```
