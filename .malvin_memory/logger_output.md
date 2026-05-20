# Malvin line logger (warning / error who tags)

TRIGGER: RepoGateOutput, Stderr, warning who, emit_repo_gate
ADVICE: `RepoGateOutput::Stderr` is an output channel only: `emit_repo_gate_line` in `src/cli/repo_checks/gate_log.rs` writes stderr with the `malvin` who tag via `print_stderr_line`. Actual warnings use `emit_repo_gate_warning` (`print_log_warning` + `[warning]` in `quality_gates.log`). Never route progress text like `Running \`kiss check\`` through `emit_repo_gate_warning`.
CONFIDENCE: 3

TRIGGER: print_log_warning, print_log_error, stderr_log
ADVICE: User-facing warnings and errors go through `src/output/stderr_log.rs` (`print_log_warning` / `print_log_error`). CLI fatal errors: `print_command_error` in `src/cli/entrypoint.rs` → `print_log_error`. Tracing WARN/ERROR: `src/tracing_init.rs` `MalvinLogLayer`. Do not duplicate severity in message text (e.g. avoid a `warning:` prefix when who is already `warning`).
CONFIDENCE: 3

TRIGGER: tracing Level, malvin_log_accepts, WARN INFO
ADVICE: In `tracing`, more severe levels compare *smaller* (`ERROR <= INFO` is true). `malvin_log_accepts_tracing_level` in `src/tracing_init.rs` uses `level <= tracing::Level::INFO` to admit INFO, WARN, and ERROR. Do not rewrite this as `level >= INFO`.
CONFIDENCE: 2

TRIGGER: who_tag_ansi, yellow red, stderr color
ADVICE: Who-tag colors live in `who_tag_ansi` (`src/output/mod.rs`): yellow for `warning`, red for `error`, cyan for others. Terminal stderr uses `format_line_with_timestamp_ansi` when `stderr_use_color()`; run logs and `quality_gates.log` use plain `format_line` without ANSI.
CONFIDENCE: 2

TRIGGER: kiss functions_per_file, gate_log, stderr_log split
ADVICE: Logger work often splits across `src/output/stderr_log.rs` (emit helpers) and `src/cli/repo_checks/gate_log.rs` (gate warnings vs progress). If `kiss check` reports `functions_per_file` on `src/output/mod.rs`, extract rather than raising limits.
CONFIDENCE: 2

TRIGGER: capture_stderr_output, logger regression test
ADVICE: Behavioral logger tests use `crate::test_stderr_capture::capture_stderr_output` in `src/cli/repo_checks/review_prep_regression.rs`. Assert `[warning]`, `[error]`, or `[malvin]` in captured output; kiss `stringify!` modules alone do not verify routing.
CONFIDENCE: 2

TRIGGER: kissconfig_warn, emit_repo_gate_warning
ADVICE: `warn_kissconfig_test_coverage_if_needed` in `src/cli/repo_checks/kissconfig_warn.rs` always calls `emit_repo_gate_warning`; its `RepoGateOutput` parameter is unused (`_output`) and does not change severity routing.
CONFIDENCE: 2
