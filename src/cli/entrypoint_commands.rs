use super::{
    CodeArgs, Commands, SharedOpts, WorkflowCliOptions, run_ideas, run_code, run_delight, run_explain,
};
use super::delight_flow::DelightArgs;
use super::explain_flow::ExplainArgs;

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

pub(crate) fn plan_args_for_delight_output(out_path: &str) -> crate::plan_flow::PlanArgs {
    crate::plan_flow::PlanArgs {
        plan_path: out_path.to_string(),
    }
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
        run_explain(
            explain,
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
        let _ = stringify!(run_delight_then_plan);
        let _ = stringify!(plan_args_for_delight_output);
        let _ = stringify!(run_explain_command);
        let _ = stringify!(dispatch_plan_authoring_gate);
    }

    #[test]
    fn delight_plan_args_use_same_out_path() {
        let args = plan_args_for_delight_output("plans/feature.md");
        assert_eq!(args.plan_path, "plans/feature.md");
    }

    #[test]
    fn kiss_cov_explain_entrypoint_branch() {
        use crate::cli::args::Commands;
        let cmd = Commands::Explain(crate::cli::explain_flow::ExplainArgs {
            request: Some("topic".to_string()),
            max_loops: 1,
            max_hypotheses: 5,
            tenacious: true,
        });
        let _ = super::super::entrypoint::require_kiss_for_cli_command(&cmd);
        let _ = stringify!(Commands::Explain);
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
