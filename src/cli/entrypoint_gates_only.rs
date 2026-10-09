use super::entrypoint::run_async_cli;
use crate::cli::{
    AgentRouteOpts, RouterOpts, SharedOpts,
    init_flow::{self, InitWorkflowOpts},
    run_tidy,
};

pub(crate) struct GatesOnlyDispatch<'a> {
    pub max_loops: usize,
    pub shared: &'a mut SharedOpts,
    pub router: &'a mut RouterOpts,
    pub matches: &'a clap::ArgMatches,
}

pub(crate) fn dispatch_gates_only_route(input: GatesOnlyDispatch<'_>) -> Result<(), String> {
    let GatesOnlyDispatch {
        mut max_loops,
        shared,
        router,
        matches,
    } = input;
    crate::cli::loop_opts::apply_default_route_tenacious(
        &mut max_loops,
        &mut shared.max_acp_retries,
        matches,
    );
    let ml = shared.ml;
    let shared = shared.clone();
    let router = router.clone();
    run_async_cli(move || async move {
        init_flow::maybe_run_init_bootstrap(InitWorkflowOpts { max_loops }, &shared, &router)
            .await?;
        crate::cli::ml_loop::run_meta_loop(ml, || {
            let shared = shared.clone();
            let router = router.clone();
            async move {
                run_tidy(
                    max_loops,
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
