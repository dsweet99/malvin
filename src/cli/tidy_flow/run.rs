use crate::cli::{SharedOpts, WorkflowCliOptions};
use crate::router_flow::{RouterArgs, run_router};

use super::effective_tidy_max_loops;

pub(crate) const TIDY_ROUTER_REQUEST: &str = "Get the gates to pass.";

#[must_use]
pub(crate) fn tidy_shared_with_gates_forced(shared: &SharedOpts) -> SharedOpts {
    let mut forced = shared.clone();
    forced.gates = true;
    forced
}

pub async fn run_tidy(
    max_loops: usize,
    max_hypotheses: usize,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let shared = tidy_shared_with_gates_forced(shared);
    run_router(
        RouterArgs {
            request: Some(TIDY_ROUTER_REQUEST.to_string()),
            max_loops: effective_tidy_max_loops(max_loops),
            max_hypotheses,
        },
        &shared,
        workflow,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kiss_cov_run_tidy_symbol() {
        let _ = run_tidy;
        let _ = TIDY_ROUTER_REQUEST;
        let _ = tidy_shared_with_gates_forced;
    }
}
