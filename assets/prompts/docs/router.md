# malvin (default route)

The default route allows up to three outer iterations (`DEFAULT_MAX_LOOPS` is 3; passing `--max-loops` is an unexpected-argument error). Each iteration starts a coder agent and sends the prompts as **one** host prompt via `start_coder_session`: `header.md`, optionally `mbc2.md` when `--creative` samples on, and `router_a.md`. The same session then receives `router_a_2.md` and waits. A lone-line `__MALVIN_DONE__` in that `router_a_2` reply skips `router_b` and `router_c`. Otherwise the same session receives `router_b.md` (creative wording via template keys) and then `router_c.md`. Malvin does not treat `__MALVIN_DONE__` in the `router_a` reply or the `router_b` reply as a stop. `router_a_2.md` asks for the marker only on an impasse (a contradiction or an obvious impossibility). `router_c.md` asks whether the request is satisfied and, if so, for that same marker. If the `router_c` reply omits it, malvin tears the agent down and starts another full iteration while the outer loop count is below `max_loops`. On the last iteration a missing marker does not run gates and does not start another iteration. With `--gates`, workspace `.malvin/gates` run after `__MALVIN_DONE__` in the `router_a_2` reply, or after `__MALVIN_DONE__` in the `router_c` reply. A pass stops the run. A failure starts another full iteration while the outer loop count is below `max_loops`, and fails the run on the last iteration. The shipped default is three iterations, so a default `--gates` run retries after a failure until the third iteration. When the iteration ends for good, `router_summarize.md` runs once on that open session.

## Summary

| | |
|---|---|
| Input | `<REQUEST>` text or existing `.md` path |
| Output | Styled stdout on a TTY (same startup chrome as other agent workflows); with `--quiet` / `-q`, direct-message bodies plus closing `TIMING` and `COST` lines; with `-qq` / `--very-quiet`, no stdout |
| Logs | `router_<n>.log` under `~/.malvinconf/logs/<hash>/<run>/` (one file per outer iteration and its coder agent; the first is `router_1.log`) |
| Requires | No `.malvin/gates` at startup (unless `--gates` later needs them) |

## Intention

Read the user request (on disk as `plan_*.md` / `{{ user_request_path }}`). `router_a.md` points at that file and at `trace.jsonl` in the run directory, and it tells the agent to check `malvin --advice` for a relevant Problems Worth Solving list (header macro `PWS`). It does not ask whether the request is already satisfied. Then send `router_a_2.md`. On an impasse the agent writes `__MALVIN_DONE__` alone on a line, and malvin skips `router_b` and `router_c`. Otherwise send `router_b.md`, then send `router_c.md` on that same session. If `router_c` omits a lone-line `__MALVIN_DONE__`, tear the agent down and start another iteration (new agent, header, and `router_a`) while iterations remain. If `router_c` emits the marker, continue to gates when `--gates` is set. A creative sample fills `{{ creative_lead }}` from `router_b_creative_lead.md` (use the MBC2 ideas while satisfying the request). `{{ satisfy_line }}` is `router_b_satisfy.md` on every work turn: satisfy the requirements, stay in scope, and, because the session is non-interactive, decide under uncertainty and record both the decision and the rationale. `{{ done_note }}` is `router_b_done_note.md` on every work turn: do not emit `__MALVIN_DONE__`. When the run ends, send `router_summarize.md` once on the already-open coder session before teardown. Do not start a new agent for summarize.

## Usage

```text
malvin [OPTION]... [REQUEST]...
```

There is no `router` subcommand. Bare `malvin REQUEST` is the default autonomous routing workflow. Multiple `REQUEST` args each run as an independent invocation (new run directory under `~/.malvinconf/logs/`, its own router run, exit `router_summarize`). If `REQUEST` is omitted (and no subcommand is given), malvin prints the command catalog on stdout and exits 0.

## Arguments

### `[REQUEST]...`

Required to run the default route. One or more shell arguments; each is a separate request (quote any argument that contains spaces). Literal text, or an existing `.md` file path.

| Form | Work directory | Stored as |
|------|----------------|-----------|
| Literal | `.` (cwd) | `plan_<random>.md` in run dir |
| `path/to/file.md` | Parent of file | `plan_<random>.md` |

## Global options

See `malvin --doc`. Notable for the default route:

| Flag | Effect |
|------|--------|
| `-g` / `--gates` | After `__MALVIN_DONE__` in the `router_a_2` reply, or after `__MALVIN_DONE__` in the `router_c` reply, run workspace `.malvin/gates`. A pass stops success and does not start another iteration. A failure starts another full router iteration while iterations remain, and fails the run on the last iteration (`DEFAULT_MAX_LOOPS` is 3, so a default run retries after a gate failure until three iterations have run). A `router_c` reply that omits the marker restarts the iteration without running gates. Also injects check text into `router_a.md` via `{{ code_extra }}`. |
| `--creative[=PROB]` | Applies only to the following REQUEST (repeatable). On each outer iteration of that request, with probability `PROB` (default `1.0` when the flag is set): include `mbc2.md` in the aggregated initial prompt (after the header), and fill `router_b.md` creative template keys for the optional work turn |
| `--quiet` / `-q` | Stdout shows direct-message bodies between `__MALVIN_DM_START__` and `__MALVIN_DM_END__` fences (without the markers), plus the closing `TIMING` and `COST` lines. Ask for a DM in the request; this flag does not select the final answer automatically. |
| `-qq` / `--very-quiet` | Suppress all process stdout, including DMs and closing `TIMING` and `COST` lines. Run-dir logs and stderr are unchanged. |

## Prompt workflow

Each outer iteration stops any already-open coder agent, drops a stored Cursor resume id, and starts a new agent. It then binds and sends one aggregated initial prompt (composition respects `--gates` and the creative sample), including `header.md`. ACP retries of the spawn delivery also create a fresh agent so the aggregated header is not re-delivered into the prior conversation.

| Turn | Piece | Role |
|------|-------|------|
| 1 (aggregated) | `header.md` (embeds workspace `AGENTS.md` via `{{ agents_insert }}` when present) + optional `mbc2.md` + `router_a.md` | One host send on a new coder agent. Header: standard Malvin context including the `__MALVIN_DM_*__` fence and optional workspace `AGENTS.md`. A direct message is for something that cannot wait until the end-of-session summary, which is itself a DM. MBC2: when `--creative` samples on. `router_a`: point at the request, the trace, and a PWS list (`malvin --advice`); optional `{{ code_extra }}` when `--gates` and `code_checks` is non-empty. |
| 2 | `router_a_2.md` | Always, on that same coder agent, after the `router_a` reply. Asks for a lone-line `__MALVIN_DONE__` only on an impasse. |
| 3 (optional) | `router_b.md` | Run only when the `router_a_2` reply did **not** emit `__MALVIN_DONE__` alone on a line; the creative sample fills `{{ creative_lead }}`; `{{ satisfy_line }}` stays `router_b_satisfy.md`; `{{ done_note }}` is always filled |
| 4 (optional) | `router_c.md` | Immediately after `router_b`, on that same session. A lone-line `__MALVIN_DONE__` means the request is satisfied and the iteration may run gates. Omitting it restarts the outer loop while iterations remain |
| Exit | `router_summarize.md` | **Once per run**, on that same already-open coder session, before teardown |

### Stop (without `--gates`)

After `router_a_2`, if any line trims to exactly `__MALVIN_DONE__`, skip `router_b` and `router_c`, send exit summarize on the open session, and stop success. Otherwise send `router_b`, then `router_c`. If `router_c` emits `__MALVIN_DONE__`, send exit summarize and stop success. If it does not, and this iteration is below `max_loops`, skip summarize, tear the agent down, and start another full iteration. On the last iteration, a missing marker still sends exit summarize and stops success. Malvin does not check the `router_a` reply or the `router_b` reply for `__MALVIN_DONE__`. Workspace gates are not run.

### Stop (with `--gates`)

Gates run after `__MALVIN_DONE__` in the `router_a_2` reply, and after `__MALVIN_DONE__` in the `router_c` reply when that turn ran. They do not run when `router_c` omits the marker. They run before the exit decision. `router_summarize.md` runs on the iteration that ends the run (a pass, a missing `router_c` marker on the last loop, or a failure on the last loop). A gate failure with loops remaining skips summarize and starts another full iteration (`router_a`, `router_a_2`, and `router_b` plus `router_c` unless `router_a_2` emitted `__MALVIN_DONE__`). A missing `router_c` marker with loops remaining does the same restart and skips gates.

| Condition | Action |
|-----------|--------|
| Gates pass | Send exit summarize on the open session, tear down, stop success. Do not start another iteration. |
| Gates fail, and this iteration is below `max_loops` | Skip exit summarize. Start another full router iteration. |
| Gates fail on the last iteration | Send exit summarize on the open session, tear down, fail with a workspace gate error |

`DEFAULT_MAX_LOOPS` is 3, so a default `malvin --gates` run takes the continue row on the first two failures and the last-iteration row on the third. The same budget applies when `router_c` omits `__MALVIN_DONE__`: the first two such iterations restart, and the third stops after summarize without running gates. The `router_a` reply and the `router_b` reply are not checked for `__MALVIN_DONE__`.

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
