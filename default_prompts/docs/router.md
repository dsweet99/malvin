# malvin (default route)

The default route runs one outer iteration (`DEFAULT_MAX_LOOPS` is 1; passing `--max-loops` is an unexpected-argument error). Malvin starts a coder agent and sends the prompts for that iteration as **one** host prompt via `start_coder_session`: `header.md`, optionally `mbc2.md` when `--creative` samples on, and `router_a.md`. The same session then receives `router_a_2.md` and waits. A lone-line `__MALVIN_DONE__` in that `router_a_2` reply skips `router_b` (and, with `--gates`, runs workspace gates before exit). Otherwise the same session receives `router_b.md` (creative wording via template keys). Malvin does not treat `__MALVIN_DONE__` in the `router_a` reply or the `router_b` reply as a stop. `router_a_2.md` asks for the marker only on an impasse (a contradiction or an obvious impossibility). When the iteration ends, `router_summarize.md` runs once on that open session.

## Summary

| | |
|---|---|
| Input | `<REQUEST>` text or existing `.md` path |
| Output | Styled stdout on a TTY (same startup chrome as other agent workflows); with `--quiet` / `-q`, only `__MALVIN_DM_*__` bodies |
| Logs | `router_1.log` under `~/.malvinconf/logs/<hash>/<run>/` (one file for the single outer iteration and its coder agent) |
| Requires | No `.malvin/gates` at startup (unless `--gates` later needs them) |

## Intention

Read the user request (on disk as `plan_*.md` / `{{ user_request_path }}`). `router_a.md` points at that file and at `trace.jsonl` in the run directory, and it tells the agent to check `malvin --advice` for a relevant Problems Worth Solving list (header macro `PWS`). It does not ask whether the request is already satisfied. Then send `router_a_2.md`. On an impasse the agent writes `__MALVIN_DONE__` alone on a line, and malvin skips `router_b`. Otherwise send `router_b.md`. A creative sample fills `{{ creative_lead }}` from `router_b_creative_lead.md` (use the MBC2 ideas while satisfying the request). `{{ satisfy_line }}` is `router_b_satisfy.md` on every work turn: satisfy the requirements, stay in scope, and, because the session is non-interactive, decide under uncertainty and record both the decision and the rationale. `{{ done_note }}` is `router_b_done_note.md` on every work turn: do not emit `__MALVIN_DONE__`. After that single iteration, send `router_summarize.md` once on the already-open coder session before teardown. Do not start a new agent for summarize.

## Usage

```text
malvin [OPTION]... [REQUEST]...
```

There is no `router` subcommand. Bare `malvin REQUEST` is the default autonomous routing workflow. Multiple `REQUEST` args each run as an independent invocation (new run directory under `~/.malvinconf/logs/`, one router pass, exit `router_summarize`). If `REQUEST` is omitted (and no subcommand is given), malvin prints the command catalog on stdout and exits 0.

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
| `-g` / `--gates` | When `router_a_2` emits `__MALVIN_DONE__`, run workspace `.malvin/gates` before exit summarize. Pass stops success; fail stops with a workspace gate error. The outer router does not start another iteration. Also injects check text into `router_a.md` via `{{ code_extra }}`. |
| `--creative[=PROB]` | Applies only to the following REQUEST (repeatable). For that request's single outer iteration, with probability `PROB` (default `1.0` when the flag is set): include `mbc2.md` in the aggregated initial prompt (after the header), and fill `router_b.md` creative template keys for the optional work turn |
| `--watch` | Before the single outer iteration, re-copy the operator request `.md` onto the run `plan_*.md` (overwrite). No-op for literal-text REQUEST |
| `--quiet` / `-q` | Stdout shows only `__MALVIN_DM_*__` bodies, plus the closing `TIMING` and `COST` lines. Plain `--do` is already DM-body-only without `--verbose`, and omits `TIMING` and `COST` unless `--verbose` |
| `--verbose` | Full prompt bodies in `prompts.log`; with `--do`, also same live agent stdout log classes as the default workflow, including `TIMING` and `COST` |

## Prompt workflow

The single outer iteration stops any already-open coder agent, drops a stored Cursor resume id, and starts a new agent. It then binds and sends one aggregated initial prompt (composition respects `--gates` and the creative sample), including `header.md`. ACP retries of the spawn delivery also create a fresh agent so the aggregated header is not re-delivered into the prior conversation.

| Turn | Piece | Role |
|------|-------|------|
| 1 (aggregated) | `header.md` (embeds workspace `AGENTS.md` via `{{ agents_insert }}` when present) + optional `mbc2.md` + `router_a.md` | One host send on a new coder agent. Header: standard Malvin context including the `__MALVIN_DM_*__` fence and optional workspace `AGENTS.md`. A direct message is for something that cannot wait until the end-of-session summary, which is itself a DM. MBC2: when `--creative` samples on. `router_a`: point at the request, the trace, and a PWS list (`malvin --advice`); optional `{{ code_extra }}` when `--gates` and `code_checks` is non-empty. |
| 2 | `router_a_2.md` | Always, on that same coder agent, after the `router_a` reply. Asks for a lone-line `__MALVIN_DONE__` only on an impasse. |
| 3 (optional) | `router_b.md` | Run only when the `router_a_2` reply did **not** emit `__MALVIN_DONE__` alone on a line; the creative sample fills `{{ creative_lead }}`; `{{ satisfy_line }}` stays `router_b_satisfy.md`; `{{ done_note }}` is always filled |
| Exit | `router_summarize.md` | **Once per run**, on that same already-open coder session, before teardown |

### Stop (without `--gates`)

After `router_a_2`, if any line trims to exactly `__MALVIN_DONE__`, skip `router_b`, send exit summarize on the open session, and stop success. Otherwise send `router_b`, then send exit summarize and stop success. Malvin does not check the `router_a` reply or the `router_b` reply for `__MALVIN_DONE__`. The outer iteration count is 1, so the run does not start another agent for `router_a`.

### Stop (with `--gates`)

Gates run **only** when `__MALVIN_DONE__` was seen in the `router_a_2` reply, and they run before `router_summarize`:

| Condition | Action |
|-----------|--------|
| Done + gates pass | Send exit summarize on the open session, tear down, stop success |
| Done + gates fail | Send exit summarize on the open session, tear down, fail with a workspace gate error |
| Not done after `router_a_2` | Send `router_b`, then send exit summarize and stop success. Gates are not run. The `router_a` reply and the `router_b` reply are not checked for `__MALVIN_DONE__`. |

### Required template keys

| Key | Required by | Value source |
|-----|-------------|--------------|
| `agents_insert` | `header.md` | Workspace root `AGENTS.md` body (labeled section), or empty when missing/blank |
| `user_request_path` | `router_a.md` | run artifacts |
| `code_extra` | `router_a.md` | `router_code_extra.md` when `--gates` and `code_checks` is non-empty (empty/whitespace `code_checks` → empty `code_extra`) |
| `creative_lead` | `router_b.md` | `router_b_creative_lead.md` when creative; else empty |
| `satisfy_line` | `router_b.md` | `router_b_satisfy.md` on every work turn, including creative |
| `done_note` | `router_b.md` | `router_b_done_note.md` on every work turn, including creative |

When the iteration ends, malvin sends `router_summarize.md` on the same coder session, then ends the session. It does not start a new agent for summarize.

## Examples

```text
malvin "Investigate flaky tests"
malvin plan.md
malvin request_1.md request_2.md
malvin --gates "Get the gates to pass"
malvin --creative notes/idea.md
malvin --creative=0.6 notes/idea.md
```
