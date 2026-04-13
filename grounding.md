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
- **Run timing** (`malvin code` and `malvin kpop`): After the workflow body Malvin writes `run_timing.json` under the run directory and prints one **stdout** summary line (timestamp-prefixed `YYYYMMDD.HHMMSS.mmm`) with wall-clock duration, cumulative LLM wait (`session/prompt`), and cumulative agent retry/backoff (sleeps between bounded retries—not model time). Durations in that line use seconds with exactly three fractional digits (for example `23.451s`). Phase buckets in the JSON match the orchestrator: Implement; Review-1 and Review-2 split into **review** vs **kpop** per attempt; Concerns; Learn when enabled. Other traces, `tracing`, and ACP logs keep their own formats; this line is only the run-timing summary.





---

# Project grounding template
For reference

## What belongs here

- **Purpose**: What the codebase is for and who it serves.
- **Long-lived constraints**: Policies and invariants that should survive refactors.
- **Stable behavioral contracts**: User-visible I/O, CLI flags, and logging semantics when they are part of the product promise—these are *not* “implementation trivia”; they are externally observable behavior you intend to keep stable.

## What does not belong here

- Ephemeral implementation details (specific internal file names, temporary workarounds). Put those in code comments, commits, or design notes.
