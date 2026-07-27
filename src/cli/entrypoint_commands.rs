use super::{
    CodeArgs, Commands, KpopArgs, SharedOpts, WorkflowCliOptions, run_inspire, run_code, run_kpop,
    run_delight, run_explain,
};
use super::delight_flow::DelightArgs;
use super::explain_flow::ExplainArgs;
use clap::ArgMatches;

use super::entrypoint::run_async_cli;
use super::entrypoint_checks::ensure_malvin_checks_for_command;

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

pub(crate) fn run_code_command(mut code: CodeArgs, shared: &SharedOpts) -> Result<(), String> {
    if code.fast {
        code.skip_pre_checks = true;
        code.trust_the_plan = true;
    }
    let requests = std::mem::take(&mut code.requests);
    let workflow = WorkflowCliOptions {
        force: !shared.no_force,
    };
    crate::sequential_requests::run_sequential("code", &requests, |request| {
        let code = code.clone();
        let shared = shared.clone();
        run_async_cli(|| run_code(code, &shared, workflow, request))
    })
}

pub(crate) fn run_kpop_command(
    mut kpop: KpopArgs,
    shared: &SharedOpts,
    _matches: &ArgMatches,
) -> Result<(), String> {
    if kpop.is_lookup() {
        return run_async_cli(|| {
            run_kpop(
                kpop,
                shared,
                WorkflowCliOptions {
                    force: !shared.no_force,
                },
            )
        });
    }
    let requests = std::mem::take(&mut kpop.requests);
    let workflow = WorkflowCliOptions {
        force: !shared.no_force,
    };
    ensure_malvin_checks_for_command(&Commands::Kpop(KpopArgs {
        max_loops: kpop.max_loops,
        max_hypotheses: kpop.max_hypotheses,
        tenacious: kpop.tenacious,
        requests: vec![],
    }))?;
    crate::sequential_requests::run_sequential("kpop", &requests, |request| {
        let kpop = kpop.with_request(request.to_string());
        let shared = shared.clone();
        run_async_cli(|| run_kpop(kpop, &shared, workflow))
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
            &mut delight,
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
        Commands::Delight(delight) => run_delight_command(delight, shared, matches),
        Commands::Explain(explain) => run_explain_command(explain, shared, matches),
        other => Err(format!("internal: unexpected plan-authoring command {other:?}")),
    }
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

#[cfg(test)]
#[path = "entrypoint_commands_tests.rs"]
mod entrypoint_commands_tests;
