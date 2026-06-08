//! Shared CLI flags for integration tests that spawn `malvin` subprocesses.

/// Keeps gate-loop integration tests fast: no tenacious budget expansion, single ACP retry.
pub const INTEGRATION_TEST_MALVIN_ARGS: &[&str] = &["--no-tenacious", "--max-acp-retries", "1"];
