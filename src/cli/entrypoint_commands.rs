use super::{
    CodeArgs, Commands, KpopArgs, SharedOpts, WorkflowCliOptions, run_inspire, run_code, run_kpop,
    run_delight, run_explain, run_revise,
};
use super::delight_flow::DelightArgs;
use super::explain_flow::ExplainArgs;
use super::revise_flow::ReviseArgs;
use clap::ArgMatches;

use super::args::Cli;
use super::bare_invoke::{bare_loop_opts, BareLoopOpts};
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
    crate::sequential_requests::run_sequential("code", &requests, |request| {
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
    crate::sequential_requests::run_sequential("", &cli.bare_args, |request| {
        let kpop = KpopArgs {
            max_loops,
            max_hypotheses: loops.max_hypotheses,
            tenacious: loops.tenacious,
            request: Some(request.to_string()),
        };
        run_async_cli(|| run_kpop(kpop, &shared, workflow))
    })
}

pub(crate) fn plan_args_for_delight_output(out_path: &str) -> crate::plan_flow::PlanArgs {
    crate::plan_flow::PlanArgs {
        plan_path: out_path.to_string(),
        out_path: out_path.to_string(),
    }
}

pub(crate) fn revise_args_for_explain_output(explain: &ExplainArgs, doc_path: &str) -> ReviseArgs {
    ReviseArgs {
        doc_path: doc_path.to_string(),
        max_loops: explain.max_loops,
        max_hypotheses: explain.max_hypotheses,
        tenacious: explain.tenacious,
    }
}

pub(crate) async fn run_explain_then_revise(
    mut explain: ExplainArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let request = explain.request.clone();
    let revise_template = revise_args_for_explain_output(&explain, "");
    run_explain(&mut explain, shared, workflow).await?;
    let request_arg = crate::cli::cli_request::require_cli_request(request.as_ref(), "explain")?;
    let doc_path = super::explain_flow::explain_revise_doc_path(&request_arg, &explain.out_path)?;
    run_revise(
        ReviseArgs {
            doc_path,
            ..revise_template
        },
        shared,
        workflow,
    )
    .await
}

pub(crate) async fn run_delight_then_plan(
    mut delight: DelightArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    run_delight(&mut delight, shared, workflow).await?;
    crate::plan_flow::run_plan(
        plan_args_for_delight_output(&delight.out_path),
        shared,
        workflow,
    )
    .await
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
        run_delight_then_plan(
            delight,
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
        Commands::Revise(revise) => run_revise_command(revise, shared, matches),
        other => Err(format!("internal: unexpected plan-authoring command {other:?}")),
    }
}

pub(crate) fn run_explain_command(
    mut explain: ExplainArgs,
    shared: &mut SharedOpts,
    matches: &clap::ArgMatches,
) -> Result<(), String> {
    super::loop_opts::apply_gate_loop_tenacious(super::loop_opts::GateLoopTenaciousApply {
        subcommand: "explain",
        max_loops: &mut explain.max_loops,
        tenacious: explain.tenacious,
        no_tenacious: shared.no_tenacious,
        max_acp_retries: &mut shared.max_acp_retries,
        matches,
    });
    run_async_cli(|| {
        run_explain_then_revise(
            explain,
            shared,
            WorkflowCliOptions {
                force: !shared.no_force,
            },
        )
    })
}

pub(crate) fn run_revise_command(
    mut revise: ReviseArgs,
    shared: &mut SharedOpts,
    matches: &clap::ArgMatches,
) -> Result<(), String> {
    super::loop_opts::apply_gate_loop_tenacious(super::loop_opts::GateLoopTenaciousApply {
        subcommand: "revise",
        max_loops: &mut revise.max_loops,
        tenacious: revise.tenacious,
        no_tenacious: shared.no_tenacious,
        max_acp_retries: &mut shared.max_acp_retries,
        matches,
    });
    run_async_cli(|| {
        run_revise(
            revise,
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
