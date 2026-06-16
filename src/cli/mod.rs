pub(crate) mod args;
mod args_bug_kpop;
mod bare_invoke;
mod bug_id_lookup;
mod bug_id_lookup_kpop;
pub(crate) mod cli_request;
pub(crate) mod command_docs;
mod commands_help;
mod commands_help_subcommand;
mod config_defaults;
mod config_loop;
pub(crate) mod entrypoint;
pub(crate) mod entrypoint_commands;
mod entrypoint_checks;
pub(crate) mod error_run_log;
pub(crate) mod exit;
pub(crate) mod kpop_flow;
pub(crate) mod models_cmd;
pub(crate) mod run_emit;
pub(crate) mod shared_opts;
pub(crate) mod tidy_flow;
pub(crate) mod delight_flow;
pub(crate) mod explain_flow;
pub(crate) mod revise_flow;

pub(crate) mod code_flow;
mod code_flow_a;
pub(crate) mod adversarial_profile;
pub(crate) mod init_discovery_flow;
mod loop_opts;
pub(crate) mod default_output_path;
pub(crate) mod workflow_kpop_shared;
pub(crate) mod kpop_summarize;

pub use crate::agent_backend::{build_agent_backend, build_agent_backend_with_tee};
pub use code_flow_a::{
    agent_io_options, build_agent, format_code_pre_check_failure, format_pre_check_gate_failure,
    format_workspace_gate_failure, new_agent_client, prepare_kpop_prompt_store,
    prepare_prompt_store, AgentStdoutTeeFlags, WorkflowCliOptions,
};
pub(crate) use code_flow::{run_code, CodeArgs};

#[cfg(test)]
mod cli_cross_cov;
#[cfg(test)]
#[path = "do_flow_tests.rs"]
mod do_flow_tests;
#[cfg(test)]
#[path = "acp_post_run_tests.rs"]
mod acp_post_run_tests;
#[cfg(test)]
#[path = "workflow_kpop_shared_tests.rs"]
mod workflow_kpop_shared_tests;
#[cfg(test)]
#[path = "kpop_summarize_tests.rs"]
pub(crate) mod kpop_summarize_tests;
#[cfg(test)]
#[path = "kpop_summarize_inline_tests.rs"]
mod kpop_summarize_inline_tests;
#[cfg(test)]
#[path = "kpop_summarize_mock_tests.rs"]
mod kpop_summarize_mock_tests;
#[cfg(test)]
#[path = "kpop_summarize_kiss_cov_tests.rs"]
mod kpop_summarize_kiss_cov_tests;
#[cfg(test)]
#[path = "models_cmd_tests.rs"]
mod models_cmd_tests;
#[cfg(test)]
mod cli_smoke_cov;
#[cfg(test)]
#[path = "cli_smoke_cov_plan.rs"]
mod cli_smoke_cov_plan;
#[cfg(test)]
mod gate_error_regression;
#[cfg(test)]
mod command_log_tests;
#[cfg(test)]
mod markdown_flag_parse_tests;

pub use crate::do_flow::run_do;
pub use crate::inspire_flow::run_inspire;
pub use crate::plan_flow::{prepare_plan_prompt_store, run_plan};
pub use args::{Cli, Commands, InspireArgs, KpopArgs, PlanArgs};
pub use config_defaults::parse_cli_with_config_defaults;
pub use entrypoint::entrypoint;
pub use exit::Exit;
pub use kpop_flow::run_kpop;
pub use run_emit::emit_run_startup_sequence;
pub use shared_opts::SharedOpts;
pub use delight_flow::run_delight;
pub use explain_flow::run_explain;
pub use revise_flow::run_revise;
pub use tidy_flow::run_tidy;
