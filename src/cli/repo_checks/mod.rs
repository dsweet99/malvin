mod command_support;
mod gate_log;
mod gate_run;
pub mod kissconfig_warn;
mod types;

#[cfg(test)]
mod review_prep_regression;
#[cfg(all(test, unix))]
mod tests_gates_common;
#[cfg(all(test, unix))]
mod tests_gates_helpers;
#[cfg(all(test, unix))]
mod tests_gates_unix;
#[cfg(all(test, unix))]
mod tests_gates_unix_extra;
#[cfg(test)]
pub use command_support::{set_fake_command_dir, FakeCommandDirGuard};

pub use gate_run::{
    run_repo_workspace_gates, run_repo_workspace_gates_no_kiss_clamp,
    run_repo_workspace_gates_with_details,
};
#[cfg(test)]
pub(crate) use types::repo_gate_failure_to_string;
pub use types::{
    GATE_FAILURE_MARKER, RepoGateCommandFailure, RepoGateFailure, RepoGateOutput,
    gate_failure_summary, is_gate_failure_error, is_pure_gate_failure_summary,
};
