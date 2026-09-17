use super::run_async_cli;
use super::super::{RouterOpts, SharedOpts, iml_loop, loop_opts, run_do, run_router};
use crate::cli::request_argv::TaggedRequest;
use crate::do_flow::DoArgs;

pub fn dispatch_do_workflow(requests: Vec<String>, shared: &SharedOpts) -> Result<(), String> {
    let iml = shared.iml;
    let shared = shared.clone();
    run_async_cli(move || async move {
        iml_loop::run_with_iml(iml, || {
            let shared = shared.clone();
            let requests = requests.clone();
            async move {
                for request in requests {
                    run_do(DoArgs { request: Some(request) }, &shared).await?;
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
    max_loops: usize,
    max_hypotheses: usize,
}

async fn run_mixed_jobs_once(pass: MixedPass<'_>) -> Result<(), String> {
    use crate::cli::request_argv::RequestKind;
    use crate::router_flow::RouterArgs;
    for job in pass.jobs {
        match job.kind {
            RequestKind::Do => {
                run_do(DoArgs { request: Some(job.text.clone()) }, pass.shared).await?;
            }
            RequestKind::Router => {
                run_router(
                    RouterArgs {
                        request: Some(job.text.clone()),
                        max_loops: pass.max_loops,
                        max_hypotheses: pass.max_hypotheses,
                    },
                    crate::cli::AgentRouteOpts {
                        shared: pass.shared,
                        router: pass.router,
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
    shared: &mut SharedOpts,
    router: &mut RouterOpts,
    matches: &clap::ArgMatches,
) -> Result<(), String> {
    let mut max_loops = router.max_loops;
    let max_hypotheses = router.max_hypotheses;
    loop_opts::apply_default_route_tenacious(&mut max_loops, &mut shared.max_acp_retries, matches);
    let iml = shared.iml;
    let shared = shared.clone();
    let router = router.clone();
    run_async_cli(move || async move {
        crate::cli::init_flow::maybe_run_init_bootstrap(
            crate::cli::init_flow::InitWorkflowOpts {
                max_loops,
                max_hypotheses,
            },
            &shared,
            &router,
        )
        .await?;
        iml_loop::run_with_iml(iml, || {
            let jobs = jobs.clone();
            let shared = shared.clone();
            let router = router.clone();
            async move {
                run_mixed_jobs_once(MixedPass {
                    jobs: &jobs,
                    shared: &shared,
                    router: &router,
                    max_loops,
                    max_hypotheses,
                })
                .await
            }
        })
        .await
    })
}

pub struct DefaultRouteDispatch<'a> {
    pub requests: Vec<String>,
    pub max_loops: usize,
    pub max_hypotheses: usize,
    pub shared: &'a mut SharedOpts,
    pub router: &'a mut RouterOpts,
    pub matches: &'a clap::ArgMatches,
}

pub fn dispatch_default_route(input: DefaultRouteDispatch<'_>) -> Result<(), String> {
    use crate::router_flow::RouterArgs;
    let DefaultRouteDispatch {
        requests,
        mut max_loops,
        max_hypotheses,
        shared,
        router,
        matches,
    } = input;
    loop_opts::apply_default_route_tenacious(
        &mut max_loops,
        &mut shared.max_acp_retries,
        matches,
    );
    let iml = shared.iml;
    let shared = shared.clone();
    let router = router.clone();
    run_async_cli(move || async move {
        crate::cli::init_flow::maybe_run_init_bootstrap(
            crate::cli::init_flow::InitWorkflowOpts {
                max_loops,
                max_hypotheses,
            },
            &shared,
            &router,
        )
        .await?;
        iml_loop::run_with_iml(iml, || {
            let requests = requests.clone();
            let shared = shared.clone();
            let router = router.clone();
            async move {
                for request in requests {
                    run_router(
                        RouterArgs {
                            request: Some(request),
                            max_loops,
                            max_hypotheses,
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
