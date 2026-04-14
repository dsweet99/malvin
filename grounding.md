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

