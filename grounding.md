# Malvin

Malvin is a Rust CLI that orchestrates non-interactive coding workflows over Cursor ACP (`agent acp`, via JSON-RPC over stdio).

## Concepts

- **Prompt templates** live in `default_prompts/` and are compiled into the `malvin` binary. `~/.malvin/prompts` is not supported. Template keys such as `{{ plan_path }}` and `**{{ quality_gates }}`** are resolved at render time, and no prompt may be sent to ACP if unresolved `{{` remains.
- **Memories** The `{{ memories }}` key is computed once per malvin command run by sampling up to 100 valid `TRIGGER:` / `ADVICE:` / `CONFIDENCE:` triples without replacement from `.malvin_memory/*.md`, weighted by `1 + CONFIDENCE`, and rendering the selected triples with one blank line between them.
- **Run artifacts** are stored under `_malvin/YYYYMMDD_HHMMSS_<id>/`. Each run records its primary inputs and outputs there, including `plan.md`, `review.md`, `result.md`, and trace logs.
- **Protected files** are `grounding.md` and `.kissconfig`. Outside the `ground` workflow, they are backed up before the first agent call and silently restored after every agent call. Agents must never edit them directly; if a task would require changing one, the agent writes `ABORT: <reason>` to `result.md`. In the `ground` workflow, `grounding.md` may be authored and refined, and `.kissconfig` is restored at the end of the workflow.
- `**kiss clamp`** runs automatically before the first agent call when source files exist but `.kissconfig` does not.
- **Quality gates (built-in)** Malvin always runs `**kiss check`**. It runs `**ruff check .**` only when Python source files exist in the workspace. It runs **pytest** only when Malvin detects Python tests (per its heuristics). It runs **Rust `cargo clippy` and `cargo test`** only when a Rust workspace applies (e.g. `Cargo.toml` present). Malvin does **not** run `**pre-commit run --all-files`** as part of quality gates; users who want pre-commit can add it as a line in `.malvin_checks`. Language and test detection for these built-ins are driven by the tree (and Malvin’s rules), not by agents inferring check lists from `grounding.md` or `.pre-commit-config.yaml`.
- `**.malvin_checks**` is an optional, user-owned overlay. If it exists, it contains one shell command per non-empty line and nothing else; Malvin runs those lines **in file order** as part of the same gate **sequence** (after the built-in checks). Malvin never creates, overwrites, edits, sorts, formats, or appends to `.malvin_checks`.
- `**{{ quality_gates }}`** Malvin computes the **full ordered list** of shell commands it will run for that gate phase (built-ins that apply, then each non-empty `.malvin_checks` line when the file exists—exact `**cargo clippy`** flags match the implementation). It uses that list for pre- and post-run execution and injects the same list into prompts so agents are not asked to re-derive which checks apply.
- **Learning** is a post-run phase for runs that are long enough to justify it (at least 5 minutes). 
- `**ground`** creates `grounding.md` with `write_grounding.md` if it does not already exist, then repeatedly runs `check_sync.md` and `improve_grounding.md` until `check_sync.md` reports `LGTM`.

## Workflows

Unless noted otherwise, a workflow consists of named prompt-template phases sent sequentially within a single ACP coder session.


| Workflow         | Pre-run quality gates | Phases                                                                                                                                                                                                                                                                                           | Post-run quality gates                             |
| ---------------- | --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------- |
| `code <request>` | Required              | Run `kiss clamp` if needed; validate the plan with `check_plan` unless `--trust-the-plan` is set; implement; run `review_1` and a `concerns` fix loop until LGTM or the `--max-loops` budget is exhausted (default 5); then do the same for `review_2`; then (optionally) run `learn`            | Required                                           |
| `sync`           | Required              | Run `kiss clamp` if needed; run `check_sync.md` and `concerns` in a loop until LGTM or the `--max-loops` budget is exhausted (default 5); then run the `review_1` and `review_2` review/fix loops; then (optionally) run `learn`                                                                 | Required                                           |
| `tidy`           | Not required          | Run `kiss clamp` if needed; run `tidy` to get the repo passing its checks; then (optionally) run `learn`                                                                                                                                                                                         | Required                                           |
| `kpop <request>` | Not required          | No `kiss clamp`; run a hypothesis-and-falsification loop, interleaving MBC2 boundary-exploration turns at a rate controlled by `--p-creative`; then (optionally) run `learn`. Display the executive summary and tl;dr in the logs and stdout logs. Total budget: `--max-hypotheses` (default 10) | Not required                                       |
| `do <request>`   | Not required          | Send one prompt and print raw output, with no review or learn phase                                                                                                                                                                                                                              | Not required                                       |
| `init`           | Not required          | Bootstrap pre-commit hooks and Git LFS configuration                                                                                                                                                                                                                                             | Not required                                       |
| `ground`         | Required              | If `grounding.md` is missing, create it with `write_grounding.md`; then run `check_sync.md`, and when it is not `LGTM`, run `improve_grounding.md`; repeat until `check_sync.md` reports `LGTM`                                                                                                  | Not required; `ground` does not change source code |


- **Review loops** work by having a reviewer write either `LGTM` or a list of issues to `review.md`. If the review is not `LGTM`, the `concerns` phase reads that file, applies fixes, and the loop repeats. Any `ABORT:` line in `result.md` stops the workflow immediately.
- **KPOP** is multi-turn. Each turn appends a new `## Step K` section to an experiment log. A `KPOP_SOLVED` marker ends the run early. MBC2 turns are meant to force structurally distant hypotheses rather than local variations.
- `header.md` is prepended before the first prompt in `code`, `sync`, `tidy`, `ground`, and `kpop`. `do_header.md` is used instead for `do`.
- **Quality gates** are the built-in checks that apply to the workspace, **in Malvin’s execution order**, plus each non-empty line of `.malvin_checks` when that file exists. The same gate sequence is used pre-run and post-run; `tidy.md` and coding rules refer to that sequence via `**{{ quality_gates }}`** and to passing those commands from the workspace root, not to rediscovering checks from prose or from `.pre-commit-config.yaml`.
- If a required post-run quality gate fails, Malvin captures the failing gate command, exit status, stdout, and stderr, sends one additional `tidy.md` prompt with those details, then reruns the post-run quality gates. If the rerun fails, the workflow fails.

## Output formatting


|                                       | code, sync, tidy, kpop, init, ground                           | do                     |
| ------------------------------------- | -------------------------------------------------------------- | ---------------------- |
| Markdown rendering                    | yes                                                            | no                     |
| Colors                                | yes; thought text is gray and directional tags are color-coded | no                     |
| Thought text on stdout                | always                                                         | only with `--thoughts` |
| Word wrap and JSON coalescing         | yes                                                            | yes                    |
| Logging headers*                      | yes                                                            | no                     |
| First log line is user's command line | yes                                                            | no                     |

## Constraints

- **JSON-RPC retry** applies to all ACP calls. Malvin makes up to 3 attempts, with some delay backoff, for transient errors such as timeouts, deadline exceeded, closed iterables, dead or zombie child processes, session initialization failures, or gRPC `[unavailable]`. Errors such as "Upgrade your plan" fail fast.
- "Upgrade your plan to continue" causes an immediate abort and explains to the user.
- The default model is "auto".
