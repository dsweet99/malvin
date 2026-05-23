mod command_support;
mod gate_log;
mod gate_run;
pub mod kissconfig_warn;
mod types;

#[cfg(test)]
mod review_prep_regression;
#[cfg(test)]
mod style_markers;
#[cfg(all(test, unix))]
mod tests_gates_common;
#[cfg(all(test, unix))]
mod tests_gates_helpers;
#[cfg(all(test, unix))]
mod tests_gates_unix;
#[cfg(all(test, unix))]
mod tests_gates_unix_extra;
#[cfg(test)]
mod tests_style;

pub(crate) use gate_log::emit_repo_gate_line;
pub use gate_run::{
    run_repo_workspace_gates, run_repo_workspace_gates_no_kiss_clamp,
    run_repo_workspace_gates_with_details,
};
pub(crate) use types::repo_gate_failure_to_string;
pub use types::{
    RepoGateCommandFailure, RepoGateFailure, RepoGateOutput, gate_failure_summary,
    is_gate_failure_error, is_pure_gate_failure_summary,
};
