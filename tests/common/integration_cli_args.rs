//! Shared CLI flags for integration tests that spawn `malvin` subprocesses.

/// Keeps gate-loop integration tests fast: no tenacious budget expansion, single ACP retry.
pub const INTEGRATION_TEST_MALVIN_ARGS: &[&str] = &["--no-tenacious", "--max-acp-retries", "1"];

/// Default hypothesis budget for gate-loop subprocess tests. Listed before per-test `extra_args` so overrides win.
pub const FAST_GATE_LOOP_TEST_ARGS: &[&str] = &["--max-hypotheses", "1"];
