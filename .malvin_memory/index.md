# Malvin memory index

Subject-specific files:

- [rust_platform_tests.md](./rust_platform_tests.md) — Linux-only integration tests, kiss attributes
- [rust_cfg_includes.md](./rust_cfg_includes.md) — `#[cfg]` + `include!`, acp memory containment on macOS
- [paths_and_gates.md](./paths_and_gates.md) — `format_prompt_path`, quality gates, protected files
- [cli_and_review.md](./cli_and_review.md) — `--no-markdown` / shared opts, cli_parity help tests, KPop review scope, kiss stringify cheats
- [logger_output.md](./logger_output.md) — `print_log_warning` / `print_log_error`, `RepoGateOutput` vs severity, tracing level filter
- [prompts_and_templates.md](./prompts_and_templates.md) — `{{ key }}` spacing, `review_plan.md`, `compose_plan_prompt`, `malformed_brace_placeholders`
- [linter_cheats.md](./linter_cheats.md) — kiss/clippy/ruff bypass inventory, `cheats.md` audit, rust-only ruff, `lib.rs` crate allows

## Workflow

TRIGGER: quality gates, malvin_checks, CI
ADVICE: Read non-empty lines from `.malvin_checks` at repo root (in order). Typical Rust layout: `kiss check`, then `cargo clippy --all-targets --all-features --` with project clippy flags, then `cargo test`. On failure, inspect `./_malvin/<run_id>/quality_gates.log` if Malvin wrote one.
CONFIDENCE: 3

TRIGGER: kiss check, kiss rules, before coding
ADVICE: Run `kiss rules` once before edits so you know thresholds (statements, attributes, nested closures). After changes, run `kiss check` before clippy/test.
CONFIDENCE: 3

TRIGGER: protected, kissconfig, malvin_checks, kissignore
ADVICE: Do not edit `.kissconfig`, `.kissignore`, or `.malvin_checks` during agent work; Malvin snapshots and restores them. Fix violations in application code instead.
CONFIDENCE: 3

TRIGGER: speak as malvin, first person
ADVICE: When Malvin invoked you, write as Malvin in first person so CLI output and agent text feel unified.
CONFIDENCE: 3
