//! Fast teardown polling helpers for integration contract tests.

pub use malvin::{
    test_post_teardown_poll_interval, test_post_teardown_wait_budget, test_wait_until_async,
};

const MALVIN_TEST_NO_REAL_AGENT_ENV: &str = "MALVIN_TEST_NO_REAL_AGENT";

pub fn enable_test_fast_teardown() {
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var(MALVIN_TEST_NO_REAL_AGENT_ENV, "1");
    }
}
