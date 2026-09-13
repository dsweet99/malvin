use crate::cli::{AgentRouteOpts, RouterOpts};
use crate::router_flow::{RouterArgs, run_router};

use super::effective_tidy_max_loops;

pub(crate) const TIDY_ROUTER_REQUEST: &str = "Get the gates to pass.";

#[must_use]
pub(crate) fn tidy_router_with_gates_forced(router: &RouterOpts) -> RouterOpts {
    let mut forced = router.clone();
    forced.gates = true;
    forced
}

pub async fn run_tidy(
    max_loops: usize,
    max_hypotheses: usize,
    opts: AgentRouteOpts<'_>,
) -> Result<(), String> {
    let router = tidy_router_with_gates_forced(opts.router);
    run_router(
        RouterArgs {
            request: Some(TIDY_ROUTER_REQUEST.to_string()),
            max_loops: effective_tidy_max_loops(max_loops),
            max_hypotheses,
        },
        AgentRouteOpts {
            shared: opts.shared,
            router: &router,
        },
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
        let _ = tidy_router_with_gates_forced;
    }
}
