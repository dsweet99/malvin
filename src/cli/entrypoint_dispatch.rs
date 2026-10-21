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
        crate::cli::init_flow::maybe_run_init_bootstrap(
            crate::cli::init_flow::InitWorkflowOpts {
                max_loops: router.max_loops,
            },
            &shared,
            &router,
        )
        .await?;
        ml_loop::run_meta_loop(ml, || {
            let jobs = jobs.clone();
            let shared = shared.clone();
            let router = router.clone();
            async move {
                for job in jobs {
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
