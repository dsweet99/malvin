use super::run_async_cli;
use crate::cli::{
    init_flow::{self, InitWorkflowOpts},
    run_init, run_tidy, WorkflowCliOptions,
};

pub(crate) fn dispatch_gates_only_route(
    mut max_loops: usize,
    max_hypotheses: usize,
    shared: &mut crate::cli::SharedOpts,
    matches: &clap::ArgMatches,
) -> Result<(), String> {
    crate::cli::loop_opts::apply_default_route_tenacious(
        &mut max_loops,
        &mut shared.max_acp_retries,
        shared.no_tenacious,
        matches,
    );
    if init_flow::should_bootstrap_gates(shared)? {
        let bootstrap_shared = init_flow::shared_for_init_bootstrap(shared);
        return run_async_cli(|| {
            run_init(
                InitWorkflowOpts {
                    max_loops,
                    max_hypotheses,
                },
                &bootstrap_shared,
                WorkflowCliOptions {
                    force: !shared.no_force,
                },
            )
        });
    }
    run_async_cli(|| {
        run_tidy(
            max_loops,
            max_hypotheses,
            shared,
            WorkflowCliOptions {
                force: !shared.no_force,
            },
        )
    })
}
