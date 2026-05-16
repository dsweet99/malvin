include!("tidy_flow/helpers.rs");

use clap::Args;
use malvin::output::{MALVIN_WHO, print_stdout_line};

#[derive(Args, Debug, Clone)]
pub struct TidyArgs {
    /// Maximum coder iterations in the tidy/review loop. Each iteration runs one coder turn (`tidy.md` on attempt 1, `tidy_concerns.md` afterwards), reviewer fan-out plus `review_write` aggregation, then workspace quality gates after LGTM. The loop exits early on LGTM plus gates pass. A value of `0` is treated as `1` (same effective semantics as `malvin code` review budgets).
    #[arg(long, default_value_t = 3)]
    pub max_loops: usize,
    #[arg(long, default_value_t = false)]
    pub no_learn: bool,
}

pub async fn run_tidy(
    tidy: TidyArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let startup = prepare_tidy_run(shared, workflow, !tidy.no_learn)?;
    let run_dir = match &startup {
        TidyStartup::SkipAgent { artifacts, .. } | TidyStartup::RunAgent { artifacts, .. } => {
            artifacts.run_dir.clone()
        }
    };
    super::error_run_log::set_command_error_run_dir(Some(run_dir));
    let r = match startup {
        TidyStartup::SkipAgent {
            artifacts,
            session_dotfile_backups,
        } => {
            merge_tidy_timing(Ok(()), &artifacts, &session_dotfile_backups)?;
            print_stdout_line(MALVIN_WHO, "DONE");
            Ok(())
        }
        TidyStartup::RunAgent {
            mut client,
            artifacts,
            session_dotfile_backups,
            store,
            context,
            run_learn,
        } => {
            async {
                let prompt = compose_tidy_prompt(&store, &context)?;
                let mut input = TidyAcpInput {
                    client: &mut client,
                    artifacts: &artifacts,
                    store: &store,
                    context: &context,
                    run_learn,
                };
                let result = run_tidy_acp(
                    &mut input,
                    prompt.trim_end(),
                    &session_dotfile_backups,
                    tidy.max_loops,
                )
                .await;
                merge_tidy_timing(result, &artifacts, &session_dotfile_backups)?;
                print_stdout_line(MALVIN_WHO, "DONE");
                Ok(())
            }
            .await
        }
    };
    if r.is_ok() {
        super::error_run_log::clear_command_error_run_dir();
    }
    r
}

mod coverage_tests;

#[cfg(test)]
#[path = "tidy_flow/helpers/tests.rs"]
mod tidy_flow_helpers_tests;
