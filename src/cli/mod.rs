pub(crate) mod admin_cmd;
pub(crate) mod args;
pub(crate) mod cli_request;
pub(crate) mod command_docs;
mod commands_help;
mod commands_help_subcommand;
mod config_defaults;
mod config_loop;
pub(crate) mod entrypoint;
mod entrypoint_checks;
pub(crate) mod entrypoint_commands;
pub(crate) mod error_run_log;
pub(crate) mod exit;
pub(crate) mod init_flow;
pub(crate) mod models_cmd;
pub(crate) mod run_emit;
pub(crate) mod shared_opts;
pub(crate) mod tidy_flow;
pub(crate) mod write_flow;

mod code_flow_a;
pub(crate) mod default_output_path;
pub(crate) mod flow_prompt_combine;
pub(crate) mod loop_opts;
pub(crate) mod one_shot_session;
pub(crate) mod workflow_router_shared;

pub use crate::agent_backend::{build_agent_backend, build_agent_backend_with_tee};
pub use code_flow_a::{
    AgentStdoutTeeFlags, WorkflowCliOptions, agent_io_options, default_workflow_stdout_tee_flags,
    format_workspace_gate_failure,
};

#[cfg(test)]
#[path = "acp_post_run_tests.rs"]
mod acp_post_run_tests;
#[cfg(test)]
#[path = "acp_post_run_timing_print_tests.rs"]
pub(crate) mod acp_post_run_timing_print_tests;
#[cfg(test)]
mod cli_cross_cov;
#[cfg(test)]
mod cli_smoke_cov;
#[cfg(test)]
mod command_log_tests;
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
pub use crate::inspire_flow::run_inspire;
pub use crate::router_flow::run_router;
pub use admin_cmd::{AdminArgs, AdminCommand, run_admin};
pub use args::{Cli, Commands, InspireArgs};
pub use config_defaults::parse_cli_with_config_defaults;
pub use entrypoint::entrypoint;
pub use exit::Exit;
pub use init_flow::run_init;
pub use loop_opts::{TENACIOUS_MAX_ACP_RETRIES, TENACIOUS_MAX_LOOPS};
pub use run_emit::emit_run_startup_sequence;
pub use shared_opts::SharedOpts;
pub use tidy_flow::run_tidy;
pub use write_flow::run_write;
