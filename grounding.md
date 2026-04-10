# Project Grounding

`malvin` is a CLI that automates implementation and review work from a user-provided plan file.

## Main Objectives

- Be easy to run: `malvin <plan-file>`.
- Execute a full implementation-plus-review cycle with minimal user coordination.
- Give users clear feedback, especially when setup or auth is missing.
- Save run artifacts and logs so results are easy to inspect.

## Core Constraints

- The workflow uses two roles (coder and reviewer) and two review phases.
- Review success is only when the reviewer output is exactly `LGTM`.
- Failed reviews trigger follow-up and retry, with a configurable max loop limit.
- Prompts are user-editable, but a fixed required prompt set must exist.
- Default prompts must be available after install.
- Each run must create a dedicated `_malvin/` run folder and copy the plan into it as `plan.md`.
- Transient agent failures are retried with bounded backoff.
- Authentication is required before execution.
- The package must be installable with a `malvin` CLI entry point.
- /implement and /concerns should run in the same agent (using --resume).
- /review_? and its /kpop should run in the same agent (using --resume), but each new review should start in a fresh agent.
- No comments or docstrings in code, except to document `click` CLI for the end user.
- Quality gates (`ruff`, `kiss`, `pytest`) must pass.

## Rust port (this repository)

- The **Python** rule above about comments targets the original `click`-based sources. The Rust implementation uses normal `//!` / `///` documentation and inline comments where they aid maintenance.
- **Quality gates** for commits: `ruff`, `pytest`, `kiss check`, `cargo clippy`, and `cargo test` (see `.pre-commit-config.yaml`).

## Scope Boundary

This grounding file defines goals and non-negotiable constraints only, not implementation details.
