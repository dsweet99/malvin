# malvin (default route)

Outer agent sessions (`--max-loops`): for each freshly created coder agent, malvin aggregates the initial prompts required by the active options—`header.md`, optionally `mbc2.md` when `--creative` samples on, and `router_a.md` (audit wording via `{{ audit_directive }}`)—and sends them as **one** host prompt via `start_coder_session`. When the outer loop continues, the next iteration stops that coder agent and starts a new one, then sends that same aggregated initial prompt again, including `header.md`. The same session then receives `router_a_2.md` and waits. A lone-line `__MALVIN_DONE__` in that `router_a_2` reply can stop the loop (optionally after `--gates` checks). Otherwise the same session receives `router_b.md` (creative wording via template keys). Malvin does not treat `__MALVIN_DONE__` in the `router_a` reply or the `router_b` reply as a stop. If the `router_a_2` reply did not emit the marker, another outer iteration may start when budget remains. When exiting, `router_summarize.md` runs once on the final open session.

## Summary

| | |
|---|---|
| Input | `<REQUEST>` text or existing `.md` path |
| Output | Styled stdout on a TTY (same startup chrome as other agent workflows); with `--quiet` / `-q`, only `__MALVIN_DM_*__` bodies |
| Logs | `router_N.log` under `~/.malvinconf/logs/<hash>/<run>/` (one file per outer iteration; N is the loop index. Each iteration has its own coder agent) |
| Requires | No `.malvin/gates` at startup (unless `--gates` later needs them) |

## Intention

Read the user request (on disk as `plan_*.md` / `{{ user_request_path }}`), ask whether requirements are still unsatisfied (`router_a.md`), then send `router_a_2.md` and either stop on `__MALVIN_DONE__` in that reply or continue with `router_b.md` (creative sample fills `{{ creative_lead }}`; `{{ satisfy_line }}` stays `router_b_satisfy.md`) to satisfy them. When the outer loop decides to exit, send `router_summarize.md` once on that final already-open coder session before teardown. Do not start a new agent for summarize. When `--max-loops` still allows and stop conditions are not met, the next outer iteration stops this agent and starts a new one for `router_a` (`header.md` is sent again). Summarize is not sent on that continue.

## Usage

```text
malvin [OPTION]... [REQUEST]...
```

There is no `router` subcommand. Bare `malvin REQUEST` is the default autonomous routing workflow. Multiple `REQUEST` args each run as an independent invocation (new run directory under `~/.malvinconf/logs/`, full outer router loop, exit `router_summarize`). If `REQUEST` is omitted (and no subcommand is given), malvin prints the command catalog on stdout and exits 0.

## Arguments

### `[REQUEST]...`

Required to run the default route. One or more shell arguments; each is a separate request (quote any argument that contains spaces). Literal text, or an existing `.md` file path (same rules as `--do`).

| Form | Work directory | Stored as |
|------|----------------|-----------|
| Literal | `.` (cwd) | `plan_<random>.md` in run dir |
| `path/to/file.md` | Parent of file | `plan_<random>.md` |

## Global options

See `malvin --doc`. Notable for the default route:

| Flag | Effect |
|------|--------|
| `--max-loops` | Outer agent-session budget (default 9999). |
| `-g` / `--gates` | When `router_a_2` emits `__MALVIN_DONE__`, run workspace `.malvin/gates`. Pass stops success; fail continues (the next outer iteration stops this agent and starts a new one). Exhausted budget with failing gates fails the run after exit summarize. Also injects check text into `router_a.md` via `{{ code_extra }}`. |
| `--creative[=PROB]` | Applies only to the following REQUEST (repeatable). Per outer iteration of that request, with probability `PROB` (default `1.0` when the flag is set): include `mbc2.md` in the aggregated initial prompt (after the header), and fill `router_b.md` creative template keys for the optional work turn |
| `--watch` | Before each outer loop, re-copy the operator request `.md` onto the run `plan_*.md` (overwrite). No-op for literal-text REQUEST |
| `--quiet` / `-q` | Stdout shows only `__MALVIN_DM_*__` bodies. Plain `--do` is already DM-body-only without `--verbose` |
| `--verbose` | Full prompt bodies in `prompts.log`; with `--do`, also same live agent stdout log classes as the default workflow |

## Prompt workflow

Each outer iteration stops any already-open coder agent, drops a stored Cursor resume id, and starts a new agent. It then binds and sends one aggregated initial prompt (composition respects `--gates` and the creative sample), including `header.md`. ACP retries of the spawn delivery also create a fresh agent so the aggregated header is not re-delivered into the prior conversation.

| Turn | Piece | Role |
|------|-------|------|
| 1 (aggregated) | `header.md` (embeds workspace `AGENTS.md` via `{{ agents_insert }}` when present) + optional `mbc2.md` + `router_a.md` (audit via `{{ audit_directive }}`) | One host send on a new coder agent. Header: standard Malvin context including the `__MALVIN_DM_*__` fence and optional workspace `AGENTS.md`. MBC2: when `--creative` samples on. `router_a`: ask whether requirements are unsatisfied; optional `{{ code_extra }}` when `--gates`. |
| 2 | `router_a_2.md` | Always, on that same coder agent, after the `router_a` reply. |
| 3 (optional) | `router_b.md` | Run only when the `router_a_2` reply did **not** emit `__MALVIN_DONE__` alone on a line; the creative sample fills `{{ creative_lead }}`; `{{ satisfy_line }}` stays `router_b_satisfy.md`; `{{ done_note }}` is always filled |
| Exit only | `router_summarize.md` | **Once per run**, when exiting the outer loop: pass to the same already-open final coder session before teardown |

### Stop / continue (without `--gates`)

After `router_a_2`, if any line trims to exactly `__MALVIN_DONE__`, skip `router_b` and stop success. Otherwise send `router_b`. Malvin does not check the `router_a` reply or the `router_b` reply for `__MALVIN_DONE__`. If outer budget remains, do not summarize yet. The next outer iteration stops this agent and starts a new one for `router_a`, including `header.md` again. Exhausting the budget without `--gates` is success (with the single exit summarize on that final session).

### Stop / continue (with `--gates`)

Gates run **only** when `__MALVIN_DONE__` was seen in the `router_a_2` reply:

| Condition | Action |
|-----------|--------|
| Done + gates pass | Send exit summarize on the open session, tear down, stop success |
| Done + gates fail, loops remain | Do not summarize. The next outer iteration stops this agent and starts a new one for `router_a` |
| Done + gates fail, budget exhausted | Send exit summarize on the open session, tear down, fail with a workspace gate error |
| Not done after `router_a_2` | Send `router_b`, then continue or exit on budget as without gates (gates not run). The `router_a` reply and the `router_b` reply are not checked for `__MALVIN_DONE__`. |

### Required template keys

| Key | Required by | Value source |
|-----|-------------|--------------|
| `agents_insert` | `header.md` | Workspace root `AGENTS.md` body (labeled section), or empty when missing/blank |
| `user_request_path` | `router_a.md` | run artifacts |
| `code_extra` | `router_a.md` | `router_code_extra.md` when `--gates` and `code_checks` is non-empty (empty/whitespace `code_checks` → empty `code_extra`) |
| `audit_directive` | `router_a.md` | `router_a_audit.md` |
| `creative_lead` | `router_b.md` | `router_b_creative_lead.md` when creative; else empty |
| `satisfy_line` | `router_b.md` | `router_b_satisfy.md` on every work turn, including creative |
| `done_note` | `router_b.md` | `router_b_done_note.md` on every work turn, including creative |

When the outer loop decides to exit, malvin sends `router_summarize.md` on the same final coder session, then ends the session. It does not start a new agent for summarize. Intermediate iterations that continue do not receive summarize; the next `router_a` starts a new agent.

## Examples

```text
malvin "Investigate flaky tests"
malvin plan.md
malvin request_1.md request_2.md
malvin --gates "Get the gates to pass"
malvin --creative --max-loops 3 notes/idea.md
malvin --creative=0.6 --max-loops 5 notes/idea.md
```
