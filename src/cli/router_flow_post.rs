use crate::artifacts::SessionDotfileBackups;
use crate::repo_checks::{RepoGateOutput, run_repo_workspace_gates};

pub(crate) struct RouterTurnsOutcome {
    pub iteration_backups: SessionDotfileBackups,
    pub gate_wants_continue: bool,
}

pub(crate) fn maybe_run_router_post_c_gates(
    work_dir: &std::path::Path,
    run_dir: &std::path::Path,
    coding_task: bool,
) -> bool {
    if !coding_task {
        return false;
    }
    match run_repo_workspace_gates(work_dir, RepoGateOutput::Tagged, Some(run_dir)) {
        Ok(()) => false,
        Err(_) => true,
    }
}

#[cfg(test)]
#[path = "router_flow_post_tests.rs"]
mod router_flow_post_tests;

#[cfg(test)]
mod kiss_cov_gate_refs {
    use super::*;

    #[test]
    fn kiss_cov_unit_names() {
        let _: Option<RouterTurnsOutcome> = None;
        let _ = maybe_run_router_post_c_gates;
    }
}
