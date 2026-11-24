use super::entrypoint::run_async_cli;
use super::{RouterOpts, SharedOpts, ml_loop, run_router};
use crate::cli::request_argv::TaggedRequest;

pub struct DefaultRouteDispatch<'a> {
    pub jobs: Vec<TaggedRequest>,
    pub shared: &'a SharedOpts,
    pub router: &'a RouterOpts,
}

pub fn dispatch_default_route(input: DefaultRouteDispatch<'_>) -> Result<(), String> {
    use crate::cli::router_flow::RouterArgs;
    let DefaultRouteDispatch {
        jobs,
        shared,
        router,
    } = input;
    let ml = shared.ml;
    let shared = shared.clone();
    let router = router.clone();
    run_async_cli(move || async move {
        crate::cli::auto_costs::maybe_run_cost_setup(&shared, &router).await;
        let mut bootstrap = router.clone();
        bootstrap.gates = jobs.iter().any(|j| j.gates.unwrap_or(router.gates));
        crate::cli::init_flow::maybe_run_init_bootstrap(
            crate::cli::init_flow::InitWorkflowOpts {
                max_loops: router.max_loops,
            },
            &shared,
            &bootstrap,
        )
        .await?;
        ml_loop::run_meta_loop(ml, || {
            let jobs = jobs.clone();
            let shared = shared.clone();
            let router = router.clone();
            async move {
                for job in jobs {
                    let mut shared = shared.clone();
                    let mut router = router.clone();
                    router.gates = job.gates.unwrap_or(router.gates);
                    if let Some(model) = job.model {
                        shared.model = model;
                    }
                    run_router(
                        RouterArgs {
                            request: Some(job.text),
                            max_loops: router.max_loops,
                        },
                        crate::cli::AgentRouteOpts {
                            shared: &shared,
                            router: &router,
                        },
                    )
                    .await?;
                }
                Ok(())
            }
        })
        .await
    })
}
