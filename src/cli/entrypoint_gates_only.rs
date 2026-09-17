use super::run_async_cli;
use crate::cli::{
    AgentRouteOpts, RouterOpts, SharedOpts,
    init_flow::{self, InitWorkflowOpts},
    run_tidy,
};

pub(crate) struct GatesOnlyDispatch<'a> {
    pub max_loops: usize,
    pub max_hypotheses: usize,
    pub shared: &'a mut SharedOpts,
    pub router: &'a mut RouterOpts,
    pub matches: &'a clap::ArgMatches,
}

pub(crate) fn dispatch_gates_only_route(input: GatesOnlyDispatch<'_>) -> Result<(), String> {
    let GatesOnlyDispatch {
        mut max_loops,
        max_hypotheses,
        shared,
        router,
        matches,
    } = input;
    crate::cli::loop_opts::apply_default_route_tenacious(
        &mut max_loops,
        &mut shared.max_acp_retries,
        matches,
    );
    let iml = shared.iml;
    let shared = shared.clone();
    let router = router.clone();
    run_async_cli(move || async move {
        init_flow::maybe_run_init_bootstrap(
            InitWorkflowOpts {
                max_loops,
                max_hypotheses,
            },
            &shared,
            &router,
        )
        .await?;
        crate::cli::iml_loop::run_with_iml(iml, || {
            let shared = shared.clone();
            let router = router.clone();
            async move {
                run_tidy(
                    max_loops,
                    max_hypotheses,
                    AgentRouteOpts {
                        shared: &shared,
                        router: &router,
                    },
                )
                .await
            }
        })
        .await
    })
}
