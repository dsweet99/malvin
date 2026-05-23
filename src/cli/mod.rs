mod args;
mod args_bug_kpop;
mod bug_flow;
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
mod mid_session_gates;
mod models_cmd;
pub(crate) mod run_emit;
mod shared_opts;
mod tidy_flow;

include!("code_flow_a.inc");
include!("code_flow_b.inc");

#[cfg(test)]
#[path = "acp_post_run_tests.rs"]
mod acp_post_run_tests;
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
pub use args::{BugArgs, Cli, CodeArgs, Commands, IdeasArgs, KpopArgs, PlanArgs};
pub use bug_flow::run_bug;
pub use entrypoint::entrypoint;
pub use exit::Exit;
pub use kpop_flow::run_kpop;
pub use run_emit::emit_run_startup_sequence;
pub use shared_opts::SharedOpts;
pub use tidy_flow::run_tidy;
pub const LEARN_MIN_ELAPSED_MS: u64 = crate::DEFAULT_LEARN_MIN_ELAPSED_MS;
