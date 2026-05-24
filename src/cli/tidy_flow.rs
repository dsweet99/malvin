use std::collections::HashMap;

use crate::artifacts::{RunArtifacts, SessionDotfileBackups};
use crate::prompts::PromptStore;
use crate::run_timing::TimingPhase;

#[must_use]
pub(crate) fn effective_tidy_max_loops(max_loops: usize) -> usize {
    max_loops.max(1)
}

#[path = "tidy_flow/prep.rs"]
mod prep;
#[path = "tidy_flow/prompt.rs"]
mod prompt;
#[path = "tidy_flow/interleaved_loop.rs"]
mod interleaved_loop;
#[cfg(test)]
#[path = "tidy_flow/test_input.rs"]
pub(crate) mod test_input;

#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use interleaved_loop::{
    run_tidy_coder_prompt_for_attempt, tidy_finish_lgtm_attempt, TidyLgtmFinishCtx,
};
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use recovery::{
    run_tidy_bonus_gate_recovery, run_tidy_concerns_coder_turn,
    run_tidy_max_loops_one_not_lgtm_recovery, run_tidy_post_concerns_recovery,
    tidy_review_attempt_with_retries, TidyMaxLoopsOneRecovery, TidyRecoveryPaths,
    TidyRecoveryRequest, TidyReviewAttemptOutcome,
};
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use run::{
    run_tidy_learn_and_summary, run_tidy_learn_prompt_if_elapsed, run_tidy_summary_prompt,
};
#[path = "tidy_flow/recovery.rs"]
pub(crate) mod recovery;
#[path = "tidy_flow/run.rs"]
mod run;
#[path = "tidy_flow/run_startup.rs"]
mod run_startup;

#[allow(unused_imports)]
pub use prep::{
    compose_tidy_concerns_prompt, compose_tidy_prompt, prepare_tidy_prompt_store,
    write_checks_do_not_pass_for_artifacts, write_checks_do_not_pass_to_review_path,
};
#[allow(unused_imports)]
pub use prompt::{run_tidy_prompt, run_tidy_prompt_with_restore};
#[allow(unused_imports)]
pub use interleaved_loop::run_tidy_interleaved_loop;
pub use run::{merge_tidy_timing, run_tidy_acp};
#[allow(unused_imports)]
pub use run_startup::{prepare_tidy_run, tidy_prompt_context};

pub enum TidyStartup {
    SkipAgent {
        artifacts: RunArtifacts,
        session_dotfile_backups: SessionDotfileBackups,
    },
    RunAgent {
        client: crate::acp::AgentClient,
        artifacts: RunArtifacts,
        session_dotfile_backups: SessionDotfileBackups,
        store: PromptStore,
        context: HashMap<String, String>,
        run_learn: bool,
    },
}

pub struct TidyAcpInput<'a> {
    pub(crate) client: &'a mut crate::acp::AgentClient,
    pub(crate) artifacts: &'a RunArtifacts,
    pub(crate) store: &'a PromptStore,
    pub(crate) context: &'a HashMap<String, String>,
    pub(crate) run_learn: bool,
}

pub struct TidyPromptRestore<'a> {
    pub(crate) prompt: &'a str,
    pub(crate) label: &'a str,
    pub(crate) phase: TimingPhase,
    pub(crate) session_dotfile_backups: &'a SessionDotfileBackups,
    pub(crate) restore_context: &'a str,
}

use crate::output::{MALVIN_WHO, print_stdout_line};
use clap::Args;

use super::{SharedOpts, WorkflowCliOptions};

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

#[cfg(test)]
#[path = "tidy_flow/tidy_flow_helpers_tests.rs"]
mod tidy_flow_helpers_tests;

