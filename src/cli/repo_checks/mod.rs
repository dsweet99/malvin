mod command_support;
mod emit;
pub mod kissconfig_warn;
mod types;
mod workspace;

#[cfg(test)]
mod style_markers;
#[cfg(test)]
mod tests_coverage;
#[cfg(test)]
mod tests_style;
#[cfg(all(test, unix))]
mod tests_gates_common;
#[cfg(all(test, unix))]
mod tests_gates_helpers;
#[cfg(all(test, unix))]
mod tests_gates_unix;
#[cfg(all(test, unix))]
mod tests_gates_unix_extra;

pub use types::{RepoGateCommandFailure, RepoGateFailure, RepoGateOutput};
pub use workspace::{
    prepare_repo_workspace, run_repo_workspace_gates, run_repo_workspace_gates_with_details,
};
