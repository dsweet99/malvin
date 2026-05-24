//! Learn-phase timing gate (crate-root leaf; no malvin internals).

/// Default learn-phase skip threshold (5 minutes), shared with CLI wiring.
pub const DEFAULT_LEARN_MIN_ELAPSED_MS: u64 = 300_000;

/// Returns true if learn should run given threshold and elapsed time.
/// Threshold of 0 means always run. Otherwise, run only if elapsed >= threshold.
#[must_use]
pub const fn should_run_learn_check(threshold_ms: u64, elapsed_ms: u64) -> bool {
    threshold_ms == 0 || elapsed_ms >= threshold_ms
}
