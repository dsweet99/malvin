use super::entrypoint::run_async_cli;
use crate::cli::{
    AgentRouteOpts, RouterOpts, SharedOpts,
    init_flow::{self, InitWorkflowOpts},
    run_tidy,
};
use malvin::config::malvin_config_file::DEFAULT_MAX_LOOPS;

pub(crate) struct GatesOnlyDispatch<'a> {
    pub shared: &'a SharedOpts,
    pub router: &'a RouterOpts,
}

pub(crate) fn dispatch_gates_only_route(input: GatesOnlyDispatch<'_>) -> Result<(), String> {
    let GatesOnlyDispatch { shared, router } = input;
    let ml = shared.ml;
    let shared = shared.clone();
    let router = router.clone();
    run_async_cli(move || async move {
        init_flow::maybe_run_init_bootstrap(
            InitWorkflowOpts {
                max_loops: DEFAULT_MAX_LOOPS,
            },
            &shared,
            &router,
        )
        .await?;
        crate::cli::ml_loop::run_meta_loop(ml, || {
            let shared = shared.clone();
            let router = router.clone();
            async move {
                run_tidy(
                    DEFAULT_MAX_LOOPS,
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
