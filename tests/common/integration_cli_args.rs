/// Shared argv prefixes for integration subprocesses (keep budgets tight).
pub const INTEGRATION_TEST_MALVIN_ARGS: &[&str] = &["--no-tenacious", "--max-acp-retries", "1"];
