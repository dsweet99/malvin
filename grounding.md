# Malvin

Malvin is a Rust CLI that orchestrates non-interactive coding workflows over Cursor ACP (`agent acp`, via JSON-RPC over stdio).

## Concepts

- **Prompt templates** live in `default_prompts/` and are compiled into the `malvin` binary. `~/.malvin/prompts` is not supported. Template keys such as `{{ plan_path }}` are resolved at render time, and no prompt may be sent to ACP if unresolved `{{` remains.
- **Run artifacts** are stored under `_malvin/YYYYMMDD_HHMMSS_<id>/`. Each run records its primary inputs and outputs there, including `plan.md`, `review.md`, `result.md`, and trace logs.
- **Protected files** are `grounding.md` and `.kissconfig`. Outside the `ground` workflow, they are backed up before the first agent call and silently restored after every agent call. Agents must never edit them directly; if a task would require changing one, the agent writes `ABORT: <reason>` to `result.md`. In the `ground` workflow, `grounding.md` may be authored and refined, and `.kissconfig` is restored at the end of the workflow.
- **`kiss clamp`** runs automatically before the first agent call when source files exist but `.kissconfig` does not.
- **Learning** is a post-run phase for runs that are long enough to justify it (at least 5 minutes). It records TRIGGER/ADVICE/CONFIDENCE triples under `.malvin_memory/`.
- **`ground`** creates `grounding.md` with `write_grounding.md` if it does not already exist, then repeatedly runs `check_sync.md` and `improve_grounding.md` until `check_sync.md` reports `LGTM`.

## Workflows

Unless noted otherwise, a workflow consists of named prompt-template phases sent sequentially within a single ACP coder session.

| Workflow | Pre-run quality gates | Phases | Post-run quality gates |
|---|---|---|---|
| `code <request>` | Required | Run `kiss clamp` if needed; validate the plan with `check_plan` unless `--trust-the-plan` is set; implement; run `review_1` and a `concerns` fix loop until LGTM or the `--max-loops` budget is exhausted (default 5); then do the same for `review_2`; then (optionally) run `learn` | Required |
| `sync` | Required | Run `kiss clamp` if needed; run `check_sync.md` and `concerns` in a loop until LGTM or the `--max-loops` budget is exhausted (default 5); then run the `review_1` and `review_2` review/fix loops; then (optionally) run `learn` | Required |
| `tidy` | Not required | Run `kiss clamp` if needed; run `tidy` to get the repo passing its checks; then (optionally) run `learn` | Required |
| `kpop <request>` | Not required | No `kiss clamp`; run a hypothesis-and-falsification loop, interleaving MBC2 boundary-exploration turns at a rate controlled by `--p-creative`; then (optionally) run `learn`. Display the executive summary and tl;dr in the logs and stdout logs. Total budget: `--max-hypotheses` (default 10) | Not required |
| `do <request>` | Not required | Send one prompt and print raw output, with no review or learn phase | Not required |
| `init` | Not required | Bootstrap pre-commit hooks and Git LFS configuration | Not required |
| `ground` | Required | If `grounding.md` is missing, create it with `write_grounding.md`; then run `check_sync.md`, and when it is not `LGTM`, run `improve_grounding.md`; repeat until `check_sync.md` reports `LGTM` | Not required; `ground` does not change source code |

- **Review loops** work by having a reviewer write either `LGTM` or a list of issues to `review.md`. If the review is not `LGTM`, the `concerns` phase reads that file, applies fixes, and the loop repeats. Any `ABORT:` line in `result.md` stops the workflow immediately.
- **KPOP** is multi-turn. Each turn appends a new `## Step K` section to an experiment log. A `KPOP_SOLVED` marker ends the run early. MBC2 turns are meant to force structurally distant hypotheses rather than local variations.
- `header.md` is prepended before the first prompt in `code`, `sync`, `tidy`, `ground`, and `kpop`. `do_header.md` is used instead for `do`.
- "quality gates" are described in `tidy.md`. The same gates are used pre-run and post-run.

## Output formatting

| | code, sync, tidy, kpop, init, ground | do |
|---|---|---|
| Markdown rendering | yes | no |
| Colors | yes; thought text is gray and directional tags are color-coded | no |
| Thought text on stdout | always | only with `--thoughts` |
| Word wrap and JSON coalescing | yes | yes |
| Logging headers* | yes | no |
| First log line is user's command line | yes | no |


## Reliability

- **JSON-RPC retry** applies to all ACP calls. Malvin makes up to 3 attempts, with some delay backoff, for transient errors such as timeouts, deadline exceeded, closed iterables, dead or zombie child processes, session initialization failures, or gRPC `[unavailable]`. Errors such as "Upgrade your plan" fail fast.




