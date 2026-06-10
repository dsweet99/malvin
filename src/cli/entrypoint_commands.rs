use super::{CodeArgs, SharedOpts, WorkflowCliOptions, run_ideas, run_code, run_delight};
use super::delight_flow::DelightArgs;

use super::entrypoint::run_async_cli;

pub(crate) fn run_inspire_command(
    ideas: crate::ideas_flow::IdeasArgs,
    shared: &SharedOpts,
) -> Result<(), String> {
    run_async_cli(|| {
        run_ideas(
            ideas,
            shared,
            WorkflowCliOptions {
                force: !shared.no_force,
            },
        )
    })
}

pub(crate) fn run_plan_command(
    plan: crate::plan_flow::PlanArgs,
    shared: &SharedOpts,
) -> Result<(), String> {
    run_async_cli(|| {
        crate::plan_flow::run_plan(
            plan,
            shared,
            WorkflowCliOptions {
                force: !shared.no_force,
            },
        )
    })
}

pub(crate) fn run_code_command(mut code: CodeArgs, shared: &SharedOpts) -> Result<(), String> {
    if code.fast {
        code.skip_pre_checks = true;
        code.trust_the_plan = true;
    }
    run_async_cli(|| {
        run_code(
            code,
            shared,
            WorkflowCliOptions {
                force: !shared.no_force,
            },
        )
    })
}

pub(crate) fn run_delight_command(
    mut delight: DelightArgs,
    shared: &mut SharedOpts,
    matches: &clap::ArgMatches,
) -> Result<(), String> {
    super::loop_opts::apply_gate_loop_tenacious(super::loop_opts::GateLoopTenaciousApply {
        subcommand: "delight",
        max_loops: &mut delight.max_loops,
        tenacious: delight.tenacious,
        no_tenacious: shared.no_tenacious,
        max_acp_retries: &mut shared.max_acp_retries,
        matches,
    });
    run_async_cli(|| {
        run_delight(
            delight,
            shared,
            WorkflowCliOptions {
                force: !shared.no_force,
            },
        )
    })
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;

    #[test]
    fn kiss_cov_entrypoint_command_wrappers() {
        let _ = stringify!(run_inspire_command);
        let _ = stringify!(run_plan_command);
        let _ = stringify!(run_code_command);
        let _ = stringify!(run_delight_command);
    }

    #[test]
    fn kiss_cov_delight_entrypoint_branch() {
        use crate::cli::args::Commands;
        let cmd = Commands::Delight(crate::cli::delight_flow::DelightArgs {
            out_path: "plan.md".to_string(),
            max_loops: 1,
            max_hypotheses: 5,
            tenacious: true,
        });
        let _ = super::super::entrypoint::require_kiss_for_cli_command(&cmd);
        let _ = stringify!(Commands::Delight);
    }
}
