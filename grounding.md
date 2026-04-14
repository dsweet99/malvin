# Malvin — grounding

## Purpose

Malvin drives a structured **implementation and review** workflow for software work: it coordinates plan-driven development and review using Cursor's **Agent Client Protocol** (`agent acp`).

## Workflows

- **`malvin code`**
- header; coding_rules; implement
- header; review_1; kpop_review; break if LGTM; concerns (check result.md for ABORT); up to max_loops times
- header; review_2; kpop_review; break if LGTM; concerns (check result.md for ABORT); up to max_loops times
- header; learn

- **`malvin kpop`**
- header
- kpop
- learn

- **`malvin do`**
- prompt

- **`malvin init`**
- Bootstraps a new project with pre-commit hooks and Git LFS configuration

## Constraints

- ACP-driven workflows must not make mutating git calls (commit, push, etc.). Read from git only.
- `malvin init` may invoke `git` for Git LFS setup.

## Repo style file

An optional `coder_style.md` file in the repo root is prepended to the first coder prompt turn when present.

## Outgoing prompts

Outgoing prompts are streamed to stdout via tee when `--no-tee` is not set. Each prompt is bracketed with a `[prompt_name...]` line. The tee output shows ACP trace lines before the bracket line.

## Run timing

After each workflow, `run_timing.json` is written to the run directory and a **stdout** summary line is printed with timing metrics.
