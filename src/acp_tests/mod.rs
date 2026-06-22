#![allow(unsafe_code, unused_imports)]

#[path = "../acp/reader_tests_coalesce_a.rs"]
pub(crate) mod reader_tests_coalesce_a;
#[path = "../acp/reader_tests_coalesce_b.rs"]
pub(crate) mod reader_tests_coalesce_b;
#[path = "../acp/reader_tests_dispatch.rs"]
pub(crate) mod reader_tests_dispatch;
#[path = "../acp/reader_tests_helpers.rs"]
pub(crate) mod reader_tests_helpers;
#[path = "../acp/reader_tests_permission.rs"]
pub(crate) mod reader_tests_permission;
#[path = "../acp/reader_tests_permission_unix.rs"]
pub(crate) mod reader_tests_permission_unix;
#[path = "../acp/reader_tests_reader_loop.rs"]
pub(crate) mod reader_tests_reader_loop;
#[path = "../acp/reader_tests_prompt_round_health.rs"]
pub(crate) mod reader_tests_prompt_round_health;
#[path = "../acp/reader_tests_retry_policy.rs"]
pub(crate) mod reader_tests_retry_policy;
#[path = "../acp/reader_tests_retry_policy_session_new.rs"]
pub(crate) mod reader_tests_retry_policy_session_new;
#[path = "../acp/reader_tests_tool_summary.rs"]
pub(crate) mod reader_tests_tool_summary;
#[path = "../acp/reader_tests_tool_summary_human.rs"]
pub(crate) mod reader_tests_tool_summary_human;
#[path = "../acp/reader_tests_tool_summary_human_bugs.rs"]
pub(crate) mod reader_tests_tool_summary_human_bugs;
#[path = "../acp/reader_tests_tool_summary_kinds.rs"]
pub(crate) mod reader_tests_tool_summary_kinds;
#[cfg(test)]
#[path = "../acp/reader_tests_tool_summary_trace_test.rs"]
pub(crate) mod reader_tests_tool_summary_trace;
#[path = "../acp/reader_tests_trace_a.rs"]
pub(crate) mod reader_tests_trace_a;
#[cfg(test)]
#[path = "../acp/reader_tests_trace_b_test.rs"]
pub(crate) mod reader_tests_trace_b;
#[cfg(test)]
#[path = "../acp/reader_tests_trace_coalesce_write_test.rs"]
pub(crate) mod reader_tests_trace_coalesce_write;
#[path = "reader_trace_coalesce_kiss_cov.rs"]
mod reader_trace_coalesce_kiss_cov;
#[cfg(test)]
#[path = "../acp/reader_tests_trace_kpop_helpers_test.rs"]
pub(crate) mod reader_tests_trace_kpop_helpers;
#[cfg(test)]
#[path = "../acp/reader_tests_trace_iterable_test.rs"]
pub(crate) mod reader_tests_trace_iterable;
#[cfg(test)]
#[path = "../acp/reader_tests_trace_upgrade_plan_test.rs"]
pub(crate) mod reader_tests_trace_upgrade_plan;

#[path = "../acp/kpop_stdout_logger_plan_helpers_kiss.rs"]
mod kpop_stdout_logger_plan_helpers_kiss;
