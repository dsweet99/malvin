# Default workflow: this branch vs `main`

Branch: `dsweet/codex` (`c72a6548`), compared to `main` (`d39d2e57`). Merge-base is `main`. This report covers the **default route** (`bare malvin REQUEST`: `header` → `kpop_common` → `router_a` → optional `router_b` → exit `router_summarize`) and the wrappers that invoke it.

## Unchanged control flow

These stop/continue rules match on both sides:

- Outer sessions are budgeted by `--max-loops` (tenacious expands to 9999 unless the flag is set on the command line).
- After `router_a`, a line that trims to exactly `__MALVIN_DONE__` skips `router_b` and can stop the run.
- Without `--gates`, exhausting the loop budget is success; `router_summarize.md` runs once on the final open session.
- With `--gates`, workspace `.malvin/gates` run **only** when `__MALVIN_DONE__` was seen. Pass → summarize and succeed. Fail with loops remaining → restart without summarize. Fail with budget exhausted → summarize and fail.

Evidence: `src/cli/router_flow_no_work.rs` (`chat_has_malvin_done`) and `src/cli/router_flow_loop_decide.rs` are the same decision table; `default_prompts/docs/router.md` stop tables are unchanged except for the `--creative` row.

`kpop_common.md` itself is byte-identical on both branches.

---

## Prompt sequence (what the agent actually receives)

Both branches still send four turns per outer session, then an exit summarize:

| Turn | `main` | this branch |
|------|--------|-------------|
| 1 | `header.md` | `header.md` (rewritten; see below) |
| 2 | `kpop_common.md` | `kpop_common.md` (same file) |
| 3 | `router_a.md` | `router_a.md` plus a Regularization paragraph |
| 4 (if not done) | `router_b.md` | `router_b.md`, or `router_b_creative.md` when `--creative` |
| Exit | `router_summarize.md` | `router_summarize.md` (unchanged) |

Evidence: `src/cli/router_flow_acp_support.rs` `run_router_turns` still calls `run_router_header`, then `build_router_kpop_common_prompt` / `run_router_kpop_common_coder_prompt`, then `router_a`, then optional `router_b`.

**Doc mismatch on this branch.** `default_prompts/docs/malvin.md` and the opening paragraph of `default_prompts/docs/router.md` say the default route is `header` → `router_a` → optional `router_b`. The prompt-workflow **table** in `router.md` still lists `kpop_common.md` as turn 2, which matches the code. The one-line summaries omit that turn.

---

## Prompt content

### `header.md`

Rewritten for a more general (non-coding-specific) voice, matching `VISION.md` on this branch. Material additions and cuts:

- New **Thinking and Reasoning** section: generate thought text “as if you have an IQ of 180”.
- Style now targets “a bright college freshman”.
- Claims-vs-hypotheses still exist, but the per-hypothesis template (Predictions / Test / Confounders) is gone from the header.
- Direct-message rule is the same idea, slightly shorter.

`{{ git_extra }}` remains in the header on both sides. `--git` still injects “You may run 'git commit'.”

### `router_a.md` / `router_b.md`

On `main`, these are short KPop commands (“Find unsatisfied requirements…”, “Satisfy the requirements.”).

On this branch both add **regularization / peak problem-solving** language:

- Prefer a redefined policy when an older analogue conflicts with a completed change-axis example.
- Classify the ambiguity, drop optional extra exclusions, choose the weakest correct interpretation.
- `router_b` further: write three complete candidate acts on independent axes, score residuals, discard any candidate that fails a named done criterion.

This matches the `VISION.md` shift: `main` names two design points (Falsification via KPop, and Regularization). This branch keeps KPop in the prompts but names **Regularization** as the main design point.

### `--creative` / `router_b_creative.md`

New on this branch. Global flag `--creative` (`src/cli/shared_opts.rs`). When the optional work turn runs, the template is `router_b_creative.md` instead of `router_b.md`. That file prepends:

```text
Run `malvin inspire PROMPT` to request creative ideas to help you satisfy the requirements.
```

then the same KPop + peak-problem-solving body as `router_b.md`.

`main` has no `--creative` flag and no `router_b_creative.md`.

### `--gates` extra (`router_code_extra.md`)

Both inject check command text via `{{ code_extra }}` into `router_a`. This branch adds:

```text
NB: The code checks may have already been run by your harness. See {{ quality_gates_log }}.
```

### Removed KPop-only prompt files

Deleted on this branch (used by older KPop gate-loop / summarize paths on `main`, not by the default router’s four turns):

- `default_prompts/kpop_block.md`
- `default_prompts/kpop_summarize.md`

The default router never sent those files as turns; they were part of the broader KPop engine that this branch removes.

---

## Wrappers that share the default router

| Command | `main` | this branch |
|---------|--------|-------------|
| `malvin tidy` | Default router, request `Get the gates to pass.`, `--gates` forced on | Same |
| `malvin init` | Separate checks-discovery KPop (`ensure_malvin_checks_discovered_for_cwd`) | Thin wrapper: render `init_constraints.md` and call `run_router`. Gains `--max-loops`, `--max-hypotheses`, `--name`, tenacious expansion. Gates **off** by default. |
| `malvin write` | Composed request into the **default router** | **Not** the default router. One agent session, two prompts (`write_a.md` then `write_b.md`). |

Evidence: `src/cli/init_flow.rs` (`run_init` → `run_router`); `src/cli/write_flow/run.rs` on this branch sends `write_a` / `write_b` coder prompts; `main`’s `write_flow/run.rs` called `run_router`.

`--quiet` docs follow that split: on `main`, quiet applied to bare REQUEST, `tidy`, and `write`. On this branch, quiet applies to bare REQUEST, `init`, and `tidy`; `write` and `inspire` are listed as one-shot tees that can also filter to DM bodies.

---

## Runtime / harness differences that affect default-route sessions

These are not new router turns, but they change how a default-route run behaves:

1. **Agent backends.** `main` is Cursor SDK (`cursor:`) and Pi (`pi:`). This branch also runs a local Codex app-server (`codex:`). Same default-route prompts; different transport.
2. **Model id brackets.** This branch accepts overrides such as `cursor:claude-opus-5[effort=high,fast=true]` or `pi:openai/gpt-5[thinking=high]`.
3. **Experiment log directory.** Gate / KPop exp logs move from `~/.malvin_home/logs/<hash>/<run>/_kpop/` on `main` to `.../_run/` on this branch (`RunArtifacts::gate_exp_log_path`). `kpop_common.md` still templates `{{ exp_log }}`; the path behind that key changed.
4. **Gate-iteration tagging.** `decide_router_gates_exit` on this branch sets `gate_loop_session::set_active_gate_iteration(Some(agent_loop))` around the gate run, then clears it. `main` did not.
5. **`max_hypotheses == 0` clamp in the kpop turn.** On `main`, `run_router_header_and_kpop` replaced `0` with `DEFAULT_MAX_HYPOTHESES` before rendering `kpop_common.md`. On this branch the value is passed through as given (`build_router_kpop_common_prompt` tuple). CLI docs still say `0` is treated as `5`; config omit-path still maps configured `0` to 5 in `apply_default_route_max_hypotheses`. An explicit `--max-hypotheses 0` on this branch can therefore render `max_hypotheses = 0` into `kpop_common.md`.
6. **Shared helper rename.** `workflow_kpop_shared.rs` (`run_kpop_workspace_gates`, KPop log line printer, extra KPop context helpers) is replaced by a smaller `workflow_router_shared.rs` (`run_router_workspace_gates`). Gate restore-then-run-then-restore behavior is the same idea; KPop-specific stdout log lines and engine loop `+1` (`kpop_engine_loop_iterations`) are gone.
7. **SDK drain idle.** Documented on this branch: per-event idle budget for Cursor/Pi (and Codex on this branch) waits, with child-health extension and bridge `progress` heartbeats. Not a prompt change; it bounds how long a default-route turn may sit silent.

---

## What is *not* a default-workflow difference

Large parts of `dsweet/codex` vs `main` are Codex integration, Pi auth filtering, coverage/kiss churn, and deleting the standalone KPop engine (`src/kpop_engine/`, `src/cli/kpop_summarize.rs`, `src/cli/checks_discovery_flow.rs`). Those matter to other commands. For the default route, the user-visible deltas are the header rewrite, regularization text, `--creative`, `init`/`write` wrapper changes, `_run` log dir, Codex as a backend, and the `max_hypotheses == 0` clamp removal in the kpop turn.

---

## Short summary

The default route is still an outer loop of `header` → `kpop_common` → `router_a` → optional `router_b` → one `router_summarize` on exit, with the same `__MALVIN_DONE__` and `--gates` rules. This branch steers the work turns toward regularization (and optional `malvin inspire` via `--creative`), rewrites `header.md`, routes `init` through the default router, takes `write` off it, stores experiment logs under `_run/`, and can talk to Codex. Docs on this branch sometimes describe the sequence as `header` → `router_a` even though `kpop_common.md` is still sent.
