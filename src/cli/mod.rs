mod args;
mod args_bug_kpop;
mod bug_flow;
mod bug_flow_remediation;
mod bug_id_lookup;
mod bug_id_lookup_kpop;
pub(crate) mod cli_request;
pub(crate) mod command_docs;
#[cfg(test)]
mod command_log_tests;
mod entrypoint;
pub(crate) mod error_run_log;
mod exit;
mod kpop_flow;
#[cfg(test)]
mod markdown_flag_parse_tests;
mod models_cmd;
pub(crate) mod run_emit;
mod shared_opts;
pub(crate) mod tidy_flow;

mod code_flow_a;
mod code_flow_b;

pub use code_flow_a::{
    agent_io_options, build_agent, format_code_pre_check_failure, format_pre_check_gate_failure,
    format_workspace_gate_failure, prepare_bug_prompt_store, prepare_hunt_kpop_prompt_store,
    prepare_kpop_prompt_store, prepare_prompt_store, AgentStdoutTeeFlags, WorkflowCliOptions,
};
pub use code_flow_b::run_code;

#[cfg(test)]
#[path = "acp_post_run_tests.rs"]
mod acp_post_run_tests;
#[cfg(test)]
#[path = "bug_flow_tests.rs"]
mod bug_flow_tests;
#[cfg(test)]
#[path = "bug_id_lookup_tests.rs"]
mod bug_id_lookup_tests;
#[cfg(test)]
#[path = "do_flow_tests.rs"]
mod do_flow_tests;
#[cfg(test)]
mod cli_cross_cov;
#[cfg(test)]
mod cli_cross_cov_kiss;
#[cfg(test)]
mod cli_smoke_cov;
#[cfg(test)]
mod gate_error_regression;

pub use crate::do_flow::run_do;
pub use crate::ideas_flow::run_ideas;
pub use crate::plan_flow::run_plan;
pub use args::{BugArgs, Cli, CodeArgs, Commands, InventArgs, KpopArgs, PlanArgs};
pub use bug_flow::run_bug;
pub use entrypoint::entrypoint;
pub use exit::Exit;
pub use kpop_flow::run_kpop;
pub use run_emit::emit_run_startup_sequence;
pub use shared_opts::SharedOpts;
pub use tidy_flow::run_tidy;
pub const LEARN_MIN_ELAPSED_MS: u64 = crate::DEFAULT_LEARN_MIN_ELAPSED_MS;
