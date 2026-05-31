#[cfg(test)]
#[path = "stdout_render_tests.rs"]
pub(super) mod stdout_render_tests;

#[cfg(test)]
#[path = "stdout_log_pair_tests.rs"]
pub(super) mod stdout_log_pair_tests;

#[cfg(test)]
#[path = "stdout_log_pair_tests_b.rs"]
pub(super) mod stdout_log_pair_tests_b;

#[cfg(test)]
pub(crate) use stdout_log_pair_tests::{
    assert_acp_tool_summary_dim_preserves_bracket, assert_tool_payload_uses_verb_styling,
};

#[cfg(test)]
#[path = "stdout_heartbeat_test_support.rs"]
pub(super) mod stdout_heartbeat_test_support;

#[cfg(test)]
#[path = "stdout_heartbeat_tests.rs"]
pub(super) mod stdout_heartbeat_tests;

#[cfg(test)]
#[path = "stdout_heartbeat_defer_tests.rs"]
pub(super) mod stdout_heartbeat_defer_tests;
