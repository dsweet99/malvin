use super::entrypoint::run_async_cli;
use super::{RouterOpts, SharedOpts, ml_loop, run_do, run_router};
use crate::cli::do_flow::DoArgs;
use crate::cli::request_argv::TaggedRequest;
use malvin::config::malvin_config_file::DEFAULT_MAX_LOOPS;

pub fn dispatch_do_workflow(requests: Vec<String>, shared: &SharedOpts) -> Result<(), String> {
    if !shared.verbose {
        malvin::run_timing::set_suppress_stdout_footnotes(true);
    }
    let ml = shared.ml;
    let shared = shared.clone();
    run_async_cli(move || async move {
        ml_loop::run_meta_loop(ml, || {
            let shared = shared.clone();
            let requests = requests.clone();
            async move {
                for request in requests {
                    run_do(
                        DoArgs {
                            request: Some(request),
                        },
                        &shared,
                    )
                    .await?;
                }
                Ok(())
            }
        })
        .await
    })
}

struct MixedPass<'a> {
    jobs: &'a [TaggedRequest],
    shared: &'a SharedOpts,
    router: &'a RouterOpts,
}

fn router_opts_for_job(base: &RouterOpts, job: &TaggedRequest) -> RouterOpts {
    base.clone().with_creative_probability(job.creative)
}

async fn run_mixed_jobs_once(pass: MixedPass<'_>) -> Result<(), String> {
    use crate::cli::request_argv::RequestKind;
    use crate::cli::router_flow::RouterArgs;
    for job in pass.jobs {
        match job.kind {
            RequestKind::Do => {
                run_do(
                    DoArgs {
                        request: Some(job.text.clone()),
                    },
                    pass.shared,
                )
                .await?;
            }
            RequestKind::Router => {
                let router = router_opts_for_job(pass.router, job);
                run_router(
                    RouterArgs {
                        request: Some(job.text.clone()),
                        max_loops: DEFAULT_MAX_LOOPS,
                    },
                    crate::cli::AgentRouteOpts {
                        shared: pass.shared,
                        router: &router,
                    },
                )
                .await?;
            }
        }
    }
    Ok(())
}

pub fn dispatch_mixed_requests(
    jobs: Vec<TaggedRequest>,
    shared: &SharedOpts,
    router: &RouterOpts,
) -> Result<(), String> {
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
                run_mixed_jobs_once(MixedPass {
                    jobs: &jobs,
                    shared: &shared,
                    router: &router,
                })
                .await
            }
        })
        .await
    })
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
