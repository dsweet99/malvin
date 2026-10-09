pub(crate) mod admin_cmd;
pub(crate) mod advice_cmd;
pub(crate) mod args;
pub(crate) mod cli_request;
pub(crate) mod command_docs;
mod commands_help;
mod config_defaults;
pub(crate) mod credits_cmd;
pub(crate) mod do_flow;
pub(crate) mod entrypoint;
mod entrypoint_checks;
pub(crate) mod error_run_log;
pub(crate) mod exit;
pub(crate) mod init_flow;
pub(crate) mod malvin_workflow;
pub(crate) mod ml_loop;
pub(crate) mod models_cmd;
pub(crate) mod remotes_cmd;
pub(crate) mod repo_checks;
pub(crate) mod request_argv;
pub(crate) mod router_flow;
pub(crate) mod run_emit;
pub(crate) mod session_header;
pub(crate) mod shared_opts;
pub(crate) mod tidy_flow;

mod code_flow_a;
pub(crate) mod loop_opts;
pub(crate) mod one_shot_session;
pub(crate) mod workflow_router_shared;

pub use code_flow_a::format_workspace_gate_failure;
pub use malvin::backends::agent_backend::AgentStdoutTeeFlags;

#[cfg(test)]
#[path = "acp_post_run_tests.rs"]
mod acp_post_run_tests;
#[cfg(test)]
#[path = "acp_post_run_timing_print_tests.rs"]
pub(crate) mod acp_post_run_timing_print_tests;
#[cfg(test)]
#[path = "bare_invoke_contract_tests.rs"]
mod bare_invoke_contract_tests;
#[cfg(test)]
mod cli_cross_cov;
#[cfg(test)]
mod cli_smoke_cov;
#[cfg(test)]
mod command_log_tests;
#[cfg(test)]
#[path = "do_flow_cli_tests.rs"]
mod do_flow_cli_tests;
#[cfg(test)]
#[path = "do_flow_tests.rs"]
mod do_flow_tests;
#[cfg(test)]
mod gate_error_regression;
#[cfg(test)]
mod markdown_flag_parse_tests;
#[cfg(test)]
#[cfg(test)]
#[cfg(test)]
#[path = "models_cmd_tests.rs"]
mod models_cmd_tests;
#[cfg(test)]
#[path = "router_flow_tests.rs"]
mod router_flow_tests;
#[cfg(test)]
#[path = "workflow_router_shared_tests.rs"]
pub(crate) mod workflow_router_shared_tests;

pub use crate::cli::do_flow::run_do;
pub use crate::cli::router_flow::run_router;
#[cfg(test)]
pub use admin_cmd::AdminArgs;
pub use admin_cmd::{AdminCommand, run_admin};
pub use args::{Cli, Commands};
#[cfg(test)]
pub use config_defaults::parse_cli_with_config_defaults;
pub use entrypoint::entrypoint;
pub use exit::Exit;
pub use shared_opts::{AgentRouteOpts, RouterOpts, SharedOpts};
pub use tidy_flow::run_tidy;

#[path = "do_flow_acp.rs"]
pub(super) mod do_flow_acp;
#[path = "do_flow_prompt.rs"]
pub(crate) mod do_flow_prompt;
#[path = "entrypoint_dispatch.rs"]
pub(super) mod entrypoint_dispatch;
#[path = "entrypoint_from.rs"]
pub(super) mod entrypoint_from;
#[path = "entrypoint_gates_only.rs"]
pub(super) mod entrypoint_gates_only;
#[path = "entrypoint_info_flags.rs"]
pub(super) mod entrypoint_info_flags;
#[path = "entrypoint_modal.rs"]
pub(super) mod entrypoint_modal;
#[path = "entrypoint_short_help.rs"]
pub(super) mod entrypoint_short_help;
#[path = "models_cmd_catalog.rs"]
pub(super) mod models_cmd_catalog;
#[path = "models_cmd_cursor.rs"]
pub(super) mod models_cmd_cursor;
#[path = "models_cmd_filter.rs"]
pub(super) mod models_cmd_filter;
#[path = "models_cmd_parse.rs"]
pub(super) mod models_cmd_parse;
#[path = "models_cmd_refresh.rs"]
pub(crate) mod models_cmd_refresh;
#[path = "models_cmd_resolve.rs"]
pub(crate) mod models_cmd_resolve;
#[path = "repo_checks/command_support.rs"]
pub(super) mod repo_checks_command_support;
#[path = "repo_checks/gate_log.rs"]
pub(super) mod repo_checks_gate_log;
#[path = "repo_checks/gate_run.rs"]
pub(super) mod repo_checks_gate_run;
#[cfg(all(test, unix))]
#[path = "repo_checks/tests_gates_common.rs"]
pub(super) mod repo_checks_tests_gates_common;
#[cfg(all(test, unix))]
#[path = "repo_checks/tests_gates_helpers.rs"]
pub(super) mod repo_checks_tests_gates_helpers;
#[cfg(all(test, unix))]
#[path = "repo_checks/tests_gates_unix.rs"]
pub(super) mod repo_checks_tests_gates_unix;
#[cfg(all(test, unix))]
#[path = "repo_checks/tests_gates_unix_extra.rs"]
pub(super) mod repo_checks_tests_gates_unix_extra;
#[path = "repo_checks/types.rs"]
pub(super) mod repo_checks_types;
#[cfg(all(test, windows))]
#[path = "repo_checks/windows_fake_command_path_tests.rs"]
pub(super) mod repo_checks_windows_fake_command_path_tests;
#[path = "router_flow_acp.rs"]
pub(crate) mod router_flow_acp;
#[path = "router_flow_acp_support.rs"]
pub(crate) mod router_flow_acp_support;
#[path = "router_flow_coder_prompts.rs"]
pub(super) mod router_flow_coder_prompts;
#[path = "router_flow_loop.rs"]
pub(crate) mod router_flow_loop;
#[path = "router_flow_loop_decide.rs"]
pub(super) mod router_flow_loop_decide;
#[path = "router_flow_no_work.rs"]
pub(crate) mod router_flow_no_work;
#[path = "router_flow_prompt.rs"]
pub(crate) mod router_flow_prompt;
#[path = "router_flow_prompt_initial.rs"]
pub(super) mod router_flow_prompt_initial;
#[path = "router_flow_prompt_summarize.rs"]
pub(super) mod router_flow_prompt_summarize;
#[path = "router_flow_prompt_turns.rs"]
pub(super) mod router_flow_prompt_turns;
#[path = "router_flow_summary_line.rs"]
pub(super) mod router_flow_summary_line;
#[path = "tidy_flow/run.rs"]
pub(super) mod tidy_flow_run;
