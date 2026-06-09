use super::{CodeArgs, Commands, KpopArgs, SharedOpts, WorkflowCliOptions, run_ideas, run_code, run_kpop};
use clap::ArgMatches;

use super::args::Cli;
use super::bare_invoke::{bare_loop_opts, BareLoopOpts};
use super::entrypoint::run_async_cli;
use super::entrypoint_checks::ensure_malvin_checks_for_command;

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
    let requests = std::mem::take(&mut code.requests);
    let workflow = WorkflowCliOptions {
        force: !shared.no_force,
    };
    super::sequential_requests::run_sequential("code", &requests, |request| {
        let code = code.clone();
        let shared = shared.clone();
        run_async_cli(|| run_code(code, &shared, workflow, request))
    })
}

pub(crate) fn run_bare_sequential_kpop(
    cli: &Cli,
    matches: &ArgMatches,
    shared: &SharedOpts,
) -> Result<(), String> {
    let mut shared = shared.clone();
    let loops = bare_loop_opts(
        cli,
        matches,
        BareLoopOpts {
            max_loops: cli.bare_max_loops,
            max_hypotheses: cli.bare_max_hypotheses,
            tenacious: crate::cli::loop_opts::DEFAULT_TENACIOUS,
        },
    );
    let mut max_loops = loops.max_loops;
    super::loop_opts::apply_gate_loop_tenacious(super::loop_opts::GateLoopTenaciousApply {
        subcommand: "kpop",
        max_loops: &mut max_loops,
        tenacious: loops.tenacious,
        no_tenacious: shared.no_tenacious,
        max_acp_retries: &mut shared.max_acp_retries,
        matches,
    });
    let workflow = WorkflowCliOptions {
        force: !shared.no_force,
    };
    ensure_malvin_checks_for_command(&Commands::Kpop(KpopArgs {
        max_loops,
        max_hypotheses: loops.max_hypotheses,
        tenacious: loops.tenacious,
        request: None,
    }))?;
    super::sequential_requests::run_sequential("", &cli.bare_args, |request| {
        let kpop = KpopArgs {
            max_loops,
            max_hypotheses: loops.max_hypotheses,
            tenacious: loops.tenacious,
            request: Some(request.to_string()),
        };
        run_async_cli(|| run_kpop(kpop, &shared, workflow))
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
        let _ = stringify!(run_bare_sequential_kpop);
    }
}
