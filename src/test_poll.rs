//! Fast polling helpers for unit and integration tests.

#[must_use]
pub fn test_post_teardown_poll_interval() -> std::time::Duration {
    if crate::acp::test_no_real_agent_enabled() {
        std::time::Duration::from_millis(1)
    } else {
        std::time::Duration::from_millis(20)
    }
}

#[must_use]
pub fn test_post_teardown_wait_budget() -> std::time::Duration {
    if crate::acp::test_no_real_agent_enabled() {
        std::time::Duration::from_millis(50)
    } else {
        std::time::Duration::from_millis(500)
    }
}

pub async fn test_wait_until_async<F>(mut condition: F) -> bool
where
    F: FnMut() -> bool,
{
    let deadline = tokio::time::Instant::now() + test_post_teardown_wait_budget();
    let poll = test_post_teardown_poll_interval();
    while tokio::time::Instant::now() < deadline {
        if condition() {
            return true;
        }
        tokio::time::sleep(poll).await;
    }
    condition()
}
