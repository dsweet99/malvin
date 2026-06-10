use super::{
    CodeArgs, Commands, SharedOpts, WorkflowCliOptions, run_inspire, run_code, run_delight, run_explain,
    run_revise,
};
use super::delight_flow::DelightArgs;
use super::explain_flow::ExplainArgs;
use super::revise_flow::ReviseArgs;

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

pub(crate) fn plan_args_for_delight_output(out_path: &str) -> crate::plan_flow::PlanArgs {
    crate::plan_flow::PlanArgs {
        plan_path: out_path.to_string(),
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
    explain: ExplainArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let out_path = explain.out_path.clone();
    let request = explain.request.clone();
    let revise_template = revise_args_for_explain_output(&explain, "");
    run_explain(explain, shared, workflow).await?;
    let request_arg = crate::cli::cli_request::require_cli_request(request.as_ref(), "explain")?;
    let doc_path = super::explain_flow::explain_revise_doc_path(&request_arg, &out_path)?;
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
    delight: DelightArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let out_path = delight.out_path.clone();
    run_delight(delight, shared, workflow).await?;
    crate::plan_flow::run_plan(plan_args_for_delight_output(&out_path), shared, workflow).await
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
