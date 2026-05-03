# Malvin

Malvin is a Rust CLI that orchestrates non-interactive coding workflows over Cursor ACP (`agent acp`, via JSON-RPC over stdio).

## Concepts

- **Prompt templates** live in `default_prompts/` and are compiled into the `malvin` binary. `~/.malvin/prompts` is not supported. Template keys such as `{{ plan_path }}` are resolved at render time, and no prompt may be sent to ACP if unresolved `{{` remains.
- **Run artifacts** are stored under `_malvin/YYYYMMDD_HHMMSS_<id>/`. Each run records its primary inputs and outputs there, including `plan.md`, `review.md`, `result.md`, and trace logs.
- **Protected files** are `grounding.md` and `.kissconfig`. They are backed up before the first agent call and silently restored after every agent call. Agents must never edit them directly; if a task would require changing one, the agent writes `ABORT: <reason>` to `result.md`.
- **`kiss clamp`** runs automatically before the first agent call when source files exist but `.kissconfig` does not.
- **Learning** is a post-run phase for runs that are long enough to justify it (at least 5 minutes). It records TRIGGER/ADVICE/CONFIDENCE triples under `.malvin_memory/`.

## Workflows

Unless noted otherwise, a workflow consists of named prompt-template phases sent sequentially within a single ACP coder session.

The `check_sync`-style loop (concerns plus `review_1` / `review_2` and optional `learn`) still lives under `malvin::orchestrator::session_flow` for tests and embedders; this CLI build does **not** expose a `malvin sync` subcommand.

| Workflow | Phases |
|---|---|
| `code <request>` | Run `kiss clamp` if needed; validate the plan with `check_plan` unless `--trust-the-plan` is set; implement; run `review_1` and a `concerns` fix loop until LGTM or the `--max-loops` budget is exhausted (default 5); then do the same for `review_2`; then (optionally) run `learn` |
| `tidy` | Run `kiss clamp` if needed; run `tidy` to get the repo passing its checks; then (optionally) run `learn` |
| `plan` | Run workspace gates; optionally write plan text to `plan.md` (or `--plan_path`); run `review_plan.md` once in a single ACP coder session |
| `kpop <request>` | Run a hypothesis-and-falsification loop, interleaving MBC2 boundary-exploration turns at a rate controlled by `--p-creative`; then (optionally) run `learn`. Total budget: `--max-hypotheses` (default 10) |
| `do <request>` | Send one prompt and print raw output, with no review or learn phase |
| `init` | Bootstrap pre-commit hooks and Git LFS configuration |

- **Review loops** work by having a reviewer write either `LGTM` or a list of issues to `review.md`. If the review is not `LGTM`, the `concerns` phase reads that file, applies fixes, and the loop repeats. Any `ABORT:` line in `result.md` stops the workflow immediately.
- **KPOP** is multi-turn. Each turn appends a new `## Step K` section to an experiment log. A `KPOP_SOLVED` marker ends the run early. MBC2 turns are meant to force structurally distant hypotheses rather than local variations.
- `header.md` is prepended before the first prompt in `code`, `tidy`, `plan`, `kpop`, and `init`. `do_header.md` is used instead for `do`.
- `coding_rules.md` is prepended to implement, review, concerns, tidy, learn, kpop, and plan prompts.

## Output formatting

| | code, tidy, plan, kpop, init | do |
|---|---|---|
| Markdown rendering | yes | no |
| Colors | yes; thought text is gray and directional tags are color-coded | no |
| Thought text on stdout | always | only with `--thoughts` |
| Word wrap and JSON coalescing | yes | yes |
| Logging headers* | yes | no |
| First log line is user's command line | yes | no |


## Reliability

- **JSON-RPC retry** applies to all ACP calls. Malvin makes up to 3 attempts, with 1 second and then 3 second backoff, when the error matches a known transient class such as timeout, deadline exceeded, closed iterables, dead or zombie child processes, session initialization failure, or gRPC `[unavailable]`. Errors of the form "Upgrade your plan" fail fast and are not retried.
- **Silent-failure retry** covers the case where a prompt succeeds at the RPC layer but the agent never produces the expected review file. In that case, Malvin retries the prompt up to 3 times with a 1 second delay. Review loops are already resilient to missing review files because a missing file is treated as non-`LGTM` and the loop continues.

## Quality gates

The implementation is only acceptable if all applicable checks pass:
- `cargo clippy --all-targets --all-features -- -D warnings` (plus pedantic/nursery/cargo)
- `cargo test`
- `kiss check`
- `ruff check` and `pytest -sv tests` (if Python files exist)


