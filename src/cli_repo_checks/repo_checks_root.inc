mod command_support {
    #![allow(clippy::missing_const_for_fn)]
    include!("command_support.inc");
}
mod emit {
    include!("emit.inc");
}
pub mod kissconfig_warn {
    include!("kissconfig_warn.inc");
}
mod types {
    include!("types.inc");
}
mod workspace {
    include!("workspace.inc");
}

#[cfg(test)]
mod style_markers {
    include!("style_markers.inc");
}
#[cfg(test)]
mod tests_coverage {
    include!("tests_coverage.inc");
}
#[cfg(all(test, unix))]
mod tests_gates_common {
    include!("tests_gates_common.inc");
}
#[cfg(all(test, unix))]
mod tests_gates_helpers {
    include!("tests_gates_helpers.inc");
}
#[cfg(all(test, unix))]
mod tests_gates_unix {
    include!("tests_gates_unix.inc");
}
#[cfg(all(test, unix))]
mod tests_gates_unix_extra {
    include!("tests_gates_unix_extra.inc");
}
#[cfg(test)]
mod tests_style {
    include!("tests_style.inc");
}

pub use types::{RepoGateCommandFailure, RepoGateFailure, RepoGateOutput};
pub use workspace::{
    run_repo_workspace_gates, run_repo_workspace_gates_no_kiss_clamp,
    run_repo_workspace_gates_with_details,
};
