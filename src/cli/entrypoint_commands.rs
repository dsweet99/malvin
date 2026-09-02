use super::write_flow::WriteArgs;
use super::{Commands, SharedOpts, WorkflowCliOptions, run_write};

use super::entrypoint::run_async_cli;

pub(crate) fn run_write_command(
    mut write_args: WriteArgs,
    shared: &mut SharedOpts,
    matches: &clap::ArgMatches,
) -> Result<(), String> {
    write_args.out_path_explicit =
        crate::cli::config_loop::subcommand_flag_from_command_line(matches, "write", "out_path");
    super::loop_opts::apply_gate_loop_tenacious(super::loop_opts::GateLoopTenaciousApply {
        subcommand: "write",
        max_loops: &mut write_args.max_loops,
        tenacious: write_args.tenacious,
        no_tenacious: shared.no_tenacious,
        max_acp_retries: &mut shared.max_acp_retries,
        matches,
    });
    run_async_cli(|| {
        run_write(
            &mut write_args,
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
        Commands::Write(write_args) => run_write_command(write_args, shared, matches),
        Commands::Admin(_) => Err(
            "internal: unexpected plan-authoring command Admin".to_string(),
        ),
    }
}

#[cfg(test)]
#[path = "entrypoint_commands_tests.rs"]
mod entrypoint_commands_tests;
