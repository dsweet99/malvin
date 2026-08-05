use super::{Commands, SharedOpts, WorkflowCliOptions, run_inspire, run_explain};
use super::explain_flow::ExplainArgs;

use super::entrypoint::run_async_cli;

pub(crate) fn run_inspire_command(
    inspire: crate::inspire_flow::InspireArgs,
    shared: &SharedOpts,
) -> Result<(), String> {
    run_async_cli(|| {
        run_inspire(
            inspire,
            shared,
            WorkflowCliOptions {
                force: !shared.no_force,
            },
        )
    })
}

pub(crate) fn run_explain_command(
    mut explain: ExplainArgs,
    shared: &mut SharedOpts,
    matches: &clap::ArgMatches,
) -> Result<(), String> {
    explain.out_path_explicit =
        crate::cli::config_loop::subcommand_flag_from_command_line(matches, "explain", "out_path");
    super::loop_opts::apply_gate_loop_tenacious(super::loop_opts::GateLoopTenaciousApply {
        subcommand: "explain",
        max_loops: &mut explain.max_loops,
        tenacious: explain.tenacious,
        no_tenacious: shared.no_tenacious,
        max_acp_retries: &mut shared.max_acp_retries,
        matches,
    });
    run_async_cli(|| {
        run_explain(
            &mut explain,
            shared,
            WorkflowCliOptions {
                force: !shared.no_force,
            },
        )
    })
}

pub(crate) fn dispatch_plan_authoring_gate(
    command: Commands,
    shared: &mut SharedOpts,
    matches: &clap::ArgMatches,
) -> Result<(), String> {
    match command {
        Commands::Explain(explain) => run_explain_command(explain, shared, matches),
        other => Err(format!("internal: unexpected plan-authoring command {other:?}")),
    }
}

#[cfg(test)]
#[path = "entrypoint_commands_tests.rs"]
mod entrypoint_commands_tests;
