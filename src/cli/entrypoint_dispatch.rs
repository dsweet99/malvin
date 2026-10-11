use super::entrypoint::run_async_cli;
use super::{RouterOpts, SharedOpts, ml_loop, run_router};
use crate::cli::request_argv::TaggedRequest;
use malvin::config::malvin_config_file::DEFAULT_MAX_LOOPS;

fn router_opts_for_job(base: &RouterOpts, job: &TaggedRequest) -> RouterOpts {
    base.clone().with_creative_probability(job.creative)
}

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
                max_loops: DEFAULT_MAX_LOOPS,
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
                    let job_router = router_opts_for_job(&router, &job);
                    run_router(
                        RouterArgs {
                            request: Some(job.text),
                            max_loops: DEFAULT_MAX_LOOPS,
                        },
                        crate::cli::AgentRouteOpts {
                            shared: &shared,
                            router: &job_router,
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
