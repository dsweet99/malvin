use super::run_async_cli;
use crate::cli::{
    WorkflowCliOptions,
    init_flow::{self, InitWorkflowOpts},
    run_tidy,
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
    run_async_cli(|| async {
        init_flow::maybe_run_init_bootstrap(
            InitWorkflowOpts {
                max_loops,
                max_hypotheses,
            },
            shared,
            WorkflowCliOptions {
                force: !shared.no_force,
            },
        )
        .await?;
        run_tidy(
            max_loops,
            max_hypotheses,
            shared,
            WorkflowCliOptions {
                force: !shared.no_force,
            },
        )
        .await
    })
}
