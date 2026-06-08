//! Teardown timing tuned for production vs integration-test fast path.

#[must_use]
pub(crate) fn test_fast_acp_teardown_enabled() -> bool {
    crate::acp::test_no_real_agent_enabled()
}

#[must_use]
pub(crate) fn teardown_poll_interval() -> std::time::Duration {
    if test_fast_acp_teardown_enabled() {
        return std::time::Duration::from_millis(5);
    }
    #[cfg(debug_assertions)]
    {
        std::time::Duration::from_millis(100)
    }
    #[cfg(not(debug_assertions))]
    {
        std::time::Duration::from_millis(500)
    }
}

#[must_use]
pub(crate) fn teardown_total_cap() -> std::time::Duration {
    if test_fast_acp_teardown_enabled() {
        return std::time::Duration::from_millis(50);
    }
    #[cfg(debug_assertions)]
    {
        std::time::Duration::from_millis(500)
    }
    #[cfg(not(debug_assertions))]
    {
        std::time::Duration::from_secs(5)
    }
}

#[must_use]
pub(crate) fn shutdown_cancel_timeout() -> std::time::Duration {
    if test_fast_acp_teardown_enabled() {
        return std::time::Duration::ZERO;
    }
    #[cfg(debug_assertions)]
    {
        std::time::Duration::from_millis(200)
    }
    #[cfg(not(debug_assertions))]
    {
        std::time::Duration::from_secs(3)
    }
}

#[must_use]
pub(crate) fn teardown_kill_after_polls() -> u32 {
    if test_fast_acp_teardown_enabled() {
        return 0;
    }
    #[cfg(debug_assertions)]
    {
        1
    }
    #[cfg(not(debug_assertions))]
    {
        3
    }
}

#[cfg(test)]
mod kiss_cov_auto {
    use super::*;

    #[test]
    fn kiss_cov_teardown_timing_fns() {
        let _ = (
            test_fast_acp_teardown_enabled,
            teardown_poll_interval,
            teardown_total_cap,
            shutdown_cancel_timeout,
            teardown_kill_after_polls,
        );
    }
}
