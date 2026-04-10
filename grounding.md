# Malvin — grounding

## Purpose

Malvin drives a structured **implementation and review** workflow for software work: it coordinates plan-driven development and review using Cursor’s **Agent Client Protocol** (`agent acp`).

## Constraints on future changes

- **Scope**: Change only what the task requires. Match existing naming, layout, and documentation tone so new code reads consistent with the rest of the project.
- **Reasoning**: Treat uncertain conclusions as hypotheses; reserve firm claims for statements you can back with evidence (code, tests, logs, or metrics).
- **Debugging**: Reproduce failures as observed, capture them with tests when appropriate, then fix—avoid speculative changes without observation.
- **Quality bar**: Treat passing the project’s automated checks (lint, tests, and project-specific validators) as part of completing a change.
- **Safety and toolchain**: Do not introduce `unsafe` Rust. Stay within the crate’s declared Rust edition and minimum supported version unless the project explicitly moves them.
