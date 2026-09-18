pub(crate) mod admin_cmd;
pub(crate) mod args;
pub(crate) mod cli_request;
pub(crate) mod command_docs;
pub(crate) mod request_argv;
mod commands_help;
mod config_defaults;
pub(crate) mod entrypoint;
mod entrypoint_checks;
pub(crate) mod error_run_log;
pub(crate) mod exit;
pub(crate) mod init_flow;
pub(crate) mod malvin_workflow;
pub(crate) mod models_cmd;
pub(crate) mod run_emit;
pub(crate) mod session_header;
pub(crate) mod iml_loop;
pub(crate) mod shared_opts;
pub(crate) mod tidy_flow;

mod code_flow_a;
pub(crate) mod loop_opts;
pub(crate) mod one_shot_session;
pub(crate) mod workflow_router_shared;

pub use code_flow_a::format_workspace_gate_failure;
pub use malvin::agent_backend::{
    AgentStdoutTeeFlags, agent_io_options, build_agent_backend, build_agent_backend_with_tee,
    default_workflow_stdout_tee_flags,
};

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
#[path = "models_cmd_auth_filter_tests.rs"]
mod models_cmd_auth_filter_tests;
#[cfg(test)]
#[path = "models_cmd_tests.rs"]
mod models_cmd_tests;
#[cfg(test)]
#[path = "router_flow_tests.rs"]
mod router_flow_tests;
#[cfg(test)]
#[path = "workflow_router_shared_tests.rs"]
pub(crate) mod workflow_router_shared_tests;

pub use crate::do_flow::run_do;
pub use crate::router_flow::run_router;
pub use admin_cmd::{AdminArgs, AdminCommand, run_admin};
pub use args::{Cli, Commands};
pub use config_defaults::parse_cli_with_config_defaults;
pub use entrypoint::entrypoint;
pub use exit::Exit;
pub use init_flow::run_init;
pub use loop_opts::TENACIOUS_MAX_LOOPS;
pub use shared_opts::{AgentRouteOpts, RouterOpts, SharedOpts};
pub use tidy_flow::run_tidy;
