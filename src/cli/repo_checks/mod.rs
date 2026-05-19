mod command_support;
mod types;
mod gate_run;
pub mod kissconfig_warn;

#[cfg(test)]
mod style_markers;
#[cfg(test)]
mod tests_coverage;
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
#[cfg(test)]
mod review_prep_regression;

pub use types::{RepoGateCommandFailure, RepoGateFailure, RepoGateOutput};
pub use gate_run::{
    run_repo_workspace_gates, run_repo_workspace_gates_no_kiss_clamp,
    run_repo_workspace_gates_with_details,
};
