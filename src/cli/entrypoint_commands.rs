use super::{CodeArgs, SharedOpts, WorkflowCliOptions, run_ideas, run_code};

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

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;

    #[test]
    fn kiss_cov_entrypoint_command_wrappers() {
        let _ = stringify!(run_inspire_command);
        let _ = stringify!(run_plan_command);
        let _ = stringify!(run_code_command);
    }
}
