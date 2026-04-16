# Malvin — grounding

## Purpose

Malvin drives a structured **implementation and review** workflow for software work: it coordinates plan-driven development and review using Cursor's **Agent Client Protocol** (`agent acp`).

## Workflows

- **`malvin code`**
- If there is existing code but no .kissconfig, run `kiss clamp`.
- header; check_plan (skip with --trust-the-plan)
- header; coding_rules; implement
- header; review_1; break if LGTM; concerns (check result.md for ABORT); up to max_loops times
- header; review_2; break if LGTM; concerns (check result.md for ABORT); up to max_loops times
- header; learn (unless the run is short)

- **`malvin kpop`**
- header
- kpop; break if agent declares success
- mbc2 between kpop blocks (rate controlled by --p-creative)
- learn (unless the run is short)
- constraint: (kpop + mbc2) <= --max_hypotheses 

- **`malvin do`**
- prompt

- **`malvin init`**
- Bootstraps a new project with pre-commit hooks and Git LFS configuration

## Other constraints
- No "documentation parity guards"