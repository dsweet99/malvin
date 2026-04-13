# Malvin — grounding

## Purpose

Malvin drives a structured **implementation and review** workflow for software work: it coordinates plan-driven development and review using Cursor’s **Agent Client Protocol** (`agent acp`).

## Workflows

- **`malvin code`**
- header
- coding_rules
- implement
- review_1; kpop review.md; break if LGTM; concerns; up to max_loops times
- review_2; kpop review.md; break if LGTM; concerns; up to max_loops times
- learn

- **`malvin kpop`**
- header
- kpop
- learn

- **`malvin do`**
- prompt

## Constraints
- Never make a writing call to git. Read from git only.
