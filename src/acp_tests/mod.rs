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
#[path = "../acp/reader_tests_retry_policy_spawn_lock.rs"]
pub(crate) mod reader_tests_retry_policy_spawn_lock;
#[path = "../acp/reader_tests_retry_policy_session_new.rs"]
pub(crate) mod reader_tests_retry_policy_session_new;
#[path = "backoff_helpers_tests.rs"]
mod backoff_helpers_tests;
#[path = "../acp/reader_tests_tool_summary.rs"]
pub(crate) mod reader_tests_tool_summary;
#[path = "../acp/reader_tests_tool_summary_human.rs"]
pub(crate) mod reader_tests_tool_summary_human;
#[path = "../acp/reader_tests_tool_summary_human_bugs.rs"]
pub(crate) mod reader_tests_tool_summary_human_bugs;
#[path = "../acp/reader_tests_tool_summary_kinds.rs"]
pub(crate) mod reader_tests_tool_summary_kinds;
#[path = "../acp/reader_tests_tool_summary_trace.rs"]
pub(crate) mod reader_tests_tool_summary_trace;
#[path = "../acp/reader_tests_trace_a.rs"]
pub(crate) mod reader_tests_trace_a;
#[path = "../acp/reader_tests_trace_b.rs"]
pub(crate) mod reader_tests_trace_b;
#[path = "../acp/reader_tests_trace_coalesce_write.rs"]
pub(crate) mod reader_tests_trace_coalesce_write;
#[cfg(test)]
#[path = "reader_trace_coalesce_kiss_cov.rs"]
mod reader_trace_coalesce_kiss_cov;
#[path = "../acp/reader_tests_trace_kpop_helpers.rs"]
pub(crate) mod reader_tests_trace_kpop_helpers;
#[path = "../acp/reader_tests_trace_iterable.rs"]
pub(crate) mod reader_tests_trace_iterable;
#[path = "../acp/reader_tests_trace_upgrade_plan.rs"]
pub(crate) mod reader_tests_trace_upgrade_plan;

#[path = "../acp/kpop_stdout_logger_plan_check.rs"]
mod kpop_stdout_logger_plan_check;
#[path = "../acp/kpop_stdout_logger_plan_check_bracket.rs"]
mod kpop_stdout_logger_plan_check_bracket;
#[path = "../acp/kpop_stdout_logger_plan_check_ext.rs"]
mod kpop_stdout_logger_plan_check_ext;
#[path = "../acp/kpop_stdout_logger_plan_check_impl.rs"]
mod kpop_stdout_logger_plan_check_impl;
#[path = "../acp/kpop_stdout_logger_plan_helpers.rs"]
mod kpop_stdout_logger_plan_helpers;
#[cfg(test)]
#[path = "../acp/kpop_stdout_logger_plan_helpers_kiss.rs"]
mod kpop_stdout_logger_plan_helpers_kiss;

#[path = "../acp/cursor_credentials_tests.rs"]
mod cursor_credentials_tests;

#[path = "../acp/deferred_log_plan_regression.rs"]
mod deferred_log_plan_regression;
