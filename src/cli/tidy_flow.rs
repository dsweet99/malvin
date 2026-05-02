include!("tidy_flow/helpers.rs");

use clap::Args;
use malvin::output::{MALVIN_WHO, print_stdout_line};

#[derive(Args, Debug, Clone)]
pub struct TidyArgs {
    #[arg(long, default_value_t = false)]
    pub no_learn: bool,
}

pub async fn run_tidy(
    tidy: TidyArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let (mut client, artifacts, grounding_backup, store, context, run_learn) =
        prepare_tidy_run(shared, workflow, !tidy.no_learn)?;
    let prompt = compose_tidy_prompt(&store, &context)?;
    let mut input = TidyAcpInput {
        client: &mut client,
        artifacts: &artifacts,
        store: &store,
        context: &context,
        run_learn,
    };
    let result = run_tidy_acp(&mut input, prompt.trim_end(), &grounding_backup).await;
    merge_tidy_timing(result, &artifacts, &grounding_backup)?;
    match crate::cli::repo_checks::run_repo_workspace_gates_with_details(
        &artifacts.work_dir,
        crate::cli::repo_checks::RepoGateOutput::Tagged,
        Some(&artifacts.run_dir),
    ) {
        Ok(()) => {}
        Err(crate::cli::repo_checks::RepoGateFailure::Command(failure)) => {
            run_tidy_prompt_after_post_run_gate_failure(
                input.client,
                &artifacts,
                &grounding_backup,
                &failure,
            )
            .await?;
            crate::cli::repo_checks::run_repo_workspace_gates(
                &artifacts.work_dir,
                crate::cli::repo_checks::RepoGateOutput::Tagged,
                Some(&artifacts.run_dir),
            )
            .map_err(|e| format!("post-run gates still failing after one tidy.md retry: {e}"))?;
        }
        Err(crate::cli::repo_checks::RepoGateFailure::Message(error)) => {
            return Err(error);
        }
    }
    print_stdout_line(MALVIN_WHO, "DONE");
    Ok(())
}

mod coverage_tests;
