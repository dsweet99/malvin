# malvin (default route)

Outer agent sessions (`--max-loops`): for each freshly created coder agent, malvin aggregates the initial prompts required by the active options—`header.md` (with KPop folded in via `{{ kpop_insert }}` from `kpop_common.md`, empty when `--no-kpop`), optionally `mbc2.md` when `--creative` samples on, and `router_a.md`—and sends them as **one** host prompt via `start_coder_session`. When the outer loop continues with the coder session kept open, or when a Cursor bridge restart resumes the same `agent_id`, later iterations skip `header.md` and send the remaining initial pieces (`kpop_common.md` when KPop is on, so the new loop's exp log is named; optional `mbc2` + `router_a`) as one aggregated follow-up turn. A lone-line `__MALVIN_DONE__` in that initial turn's reply can stop the loop (optionally after `--gates` checks). Otherwise the same session receives `router_b.md` (or `router_b_creative.md` when that iteration sampled creative), and another outer iteration may start when budget remains. When exiting, `router_summarize.md` runs once on the final open session.

## Summary

| | |
|---|---|
| Input | `<REQUEST>` text or existing `.md` path |
| Output | Styled stdout on a TTY (same startup chrome as other agent workflows); with `--quiet` / `-q`, only `__MALVIN_DM_*__` bodies |
| Logs | `router_N.log` under `~/.malvin_home/logs/<hash>/<run>/` (one file per outer iteration; N is the loop index. Continue keeps the coder session open, so several files can share one session) |
| Requires | No `.malvin/gates` at startup (unless `--gates` later needs them) |

## Intention

Read the user request (on disk as `plan_*.md` / `{{ user_request_path }}`), ask whether requirements are still unsatisfied (`router_a.md`), and either stop on `__MALVIN_DONE__` or continue with `router_b.md` (or `router_b_creative.md` when `--creative` samples on) to satisfy them. When the outer loop decides to exit, send `router_summarize.md` once on that final already-open coder session before teardown. When `--max-loops` still allows and stop conditions are not met, continue with another outer iteration on the **same** kept-open coder session (no tear-down; `header.md` is not re-sent).

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
| `--max-loops` | Outer agent-session budget (default 9999). |
| `--max-hypotheses` | Hypothesis budget (default 5). When omitted, `[default_workflow].max_hypotheses` is used. Explicit CLI wins over config. |
| `-g` / `--gates` | When `router_a` emits `__MALVIN_DONE__`, run workspace `.malvin/gates`. Pass stops success; fail continues (next outer iteration on the kept-open session). Exhausted budget with failing gates fails the run after exit summarize. Also injects check text into `router_a.md` via `{{ code_extra }}`. |
| `--creative[=PROB]` | Per outer iteration, with probability `PROB` (default `1.0` when the flag is set): include `mbc2.md` in the aggregated initial prompt (after header / kpop insert), and use `router_b_creative.md` for the optional work turn |
| `--watch` | Before each outer loop, re-copy the operator request `.md` onto the run `plan_*.md` (overwrite). No-op for literal-text REQUEST |
| `--quiet` / `-q` | Stdout shows only `__MALVIN_DM_*__` bodies. Plain `--do` is already DM-body-only without `--verbose` |
| `--verbose` | Full prompt bodies in `prompts.log`; with `--do`, also same live agent stdout log classes as the default workflow |

## Prompt workflow

Each outer iteration ensures one coder session. On a **fresh** agent, malvin binds and sends one aggregated initial prompt (composition respects `no_kpop`, `--gates`, and the creative sample). Reused sessions and Cursor resumes of the same `agent_id` omit `header.md` from that aggregate; when KPop is on, they still include standalone `kpop_common.md` (so the new loop's exp log is named) plus optional `mbc2` + `router_a`. ACP retries of the spawn delivery create a fresh agent so the aggregated header is not re-delivered into the prior conversation.

| Turn | Piece | Role |
|------|-------|------|
| 1 (aggregated) | Fresh: `header.md` (embeds `kpop_common.md` via `{{ kpop_insert }}`, empty under `--no-kpop`; embeds workspace `AGENTS.md` via `{{ agents_insert }}` when present) + optional `mbc2.md` + `router_a.md`. Reused/resume: omit `header.md`; when KPop is on, send `kpop_common.md` + optional `mbc2.md` + `router_a.md`. | One host send. Header (fresh only): standard Malvin context including the `__MALVIN_DM_*__` fence, optional KPop method, and optional workspace `AGENTS.md`. MBC2: when `--creative` samples on. `router_a`: ask whether requirements are unsatisfied; optional `{{ code_extra }}` when `--gates`. |
| 2 (optional) | `router_b.md` or `router_b_creative.md` | Run only when the aggregated initial turn did **not** emit `__MALVIN_DONE__` alone on a line; creative sample selects `router_b_creative.md` |
| Exit only | `router_summarize.md` | **Once per run**, when exiting the outer loop: pass to the same already-open final coder session before teardown |

### Stop / continue (without `--gates`)

After the aggregated initial turn, if any line trims to exactly `__MALVIN_DONE__`, skip `router_b` and stop success. Otherwise send `router_b` and, if outer budget remains, keep the coder session open (no tear-down, no summarize) and run another outer iteration—`header.md` is not re-sent. Exhausting the budget without `--gates` is success (with the single exit summarize on that final session).

### Stop / continue (with `--gates`)

Gates run **only** when `__MALVIN_DONE__` was seen:

| Condition | Action |
|-----------|--------|
| Done + gates pass | Send exit summarize on the open session, tear down, stop success |
| Done + gates fail, loops remain | Keep session open (no summarize); next outer iteration omits `header.md` |
| Done + gates fail, budget exhausted | Send exit summarize on the open session, tear down, fail with a workspace gate error |
| Not done | Send `router_b`; continue or exit on budget as without gates (gates not run) |

### Required template keys

| Key | Required by | Value source |
|-----|-------------|--------------|
| `kpop_insert` | `header.md` | Rendered `kpop_common.md` (router, when KPop on); empty string when `--no-kpop` or non-router header consumers |
| `agents_insert` | `header.md` | Workspace root `AGENTS.md` body (labeled section), or empty when missing/blank |
| `user_request_path` | `router_a.md` | run artifacts |
| `code_extra` | `router_a.md` | `router_code_extra.md` when `--gates` and `code_checks` is non-empty (empty/whitespace `code_checks` → empty `code_extra`) |

When the outer loop decides to exit, malvin sends `router_summarize.md` on the same final coder session, then ends the session. Intermediate iterations that continue keep the session open and do not receive summarize.

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
malvin --creative=0.6 --max-loops 5 notes/idea.md
```
