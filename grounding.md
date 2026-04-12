# Malvin — grounding

## Purpose

Malvin drives a structured **implementation and review** workflow for software work: it coordinates plan-driven development and review using Cursor’s **Agent Client Protocol** (`agent acp`).

## Constraints on future changes

- **Scope**: Change only what the task requires. Match existing naming, layout, and documentation tone so new code reads consistent with the rest of the project.
- **Reasoning**: Treat uncertain conclusions as hypotheses; reserve firm claims for statements you can back with evidence (code, tests, logs, or metrics).
- **Debugging**: Reproduce failures as observed, capture them with tests when appropriate, then fix—avoid speculative changes without observation.
- **Quality bar**: Treat passing the project’s automated checks (lint, tests, and project-specific validators) as part of completing a change.
- **Safety and toolchain**: Do not introduce `unsafe` Rust in **non-test** code. Test-only code may use `unsafe` when the standard library requires it (for example environment-variable fixtures), kept localized and gated with explicit `#[allow(unsafe_code)]` on the smallest enclosing test module. Stay within the crate’s declared Rust edition and minimum supported version unless the project explicitly moves them.
- **Tee** (`--no-tee` to disable): When tee is on, the primary plan/request document, the recorded invocation line (`Command: …`) printed at startup, and ACP session log content are echoed to stdout. Trace files on disk still begin with the same `Command: …` prelude when applicable; stdout tee of a trace skips repeating that prelude so the invocation line is not shown twice. With tee off, those streams are not printed to stdout; run-directory files (for example `command.log` and trace logs) are still written for inspection.
- **Progress and post-run metrics (stable):** Workflow phase lines (for example `Implement`, `Review-1`), the `Logs: …` line, and `DONE` are printed to **stdout** even when `--no-tee` is set—only the tee-gated items above are suppressed. Edit-efficiency metering is not recorded in this build; a short “not measured” hint uses **`eprintln!` (stderr)** after the run so it remains visible if stdout is piped or captured alone.
