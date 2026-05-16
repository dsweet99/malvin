use crate::acp::{AgentClient, AgentError, CoderPromptOptions};
use crate::artifacts::{RunArtifacts, SessionDotfileBackups};
use crate::prompts::{PromptError, PromptStore};
use crate::run_timing::{self, RunTiming, TimingPhase};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::{Arc, Mutex};

mod memory_context;

include!("helpers.rs");

#[cfg(test)]
mod helpers_tests;

mod constants;
mod review_attempt_kernel;
mod review_fanout_desc;
mod review_fanout_run;
mod review_fanout_write;
mod review_loop_helpers;

pub use review_attempt_kernel::{
    ReviewAttemptKernelInput, load_review_descriptions_for_kernel, review_attempt_is_lgtm,
    run_review_fanout_prefix,
};

mod check_plan;
mod review_loop;
pub mod session_flow;

mod bug_remediation;

use session_flow::{run_coder_session_summary_only, run_coder_session_until_pre_summary};

use workflow_context as workflow_context_inner;

pub type PreSummaryMidFn =
    for<'a> fn(
        &'a mut AgentClient,
        &'a RunArtifacts,
        &'a SessionDotfileBackups,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + 'a>>;

fn mid_noop<'a>(
    _: &'a mut AgentClient,
    _: &'a RunArtifacts,
    _: &'a SessionDotfileBackups,
) -> Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + 'a>> {
    Box::pin(async { Ok(()) })
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct WorkflowError(pub String);

pub(crate) fn prefer_primary_errors_over_timing(
    workflow_result: Result<(), WorkflowError>,
    end_result: Result<(), WorkflowError>,
    timing_result: Result<(), WorkflowError>,
) -> Result<(), WorkflowError> {
    if matches!((&workflow_result, &end_result), (Ok(()), Ok(()))) {
        timing_result
    } else {
        let _ = timing_result;
        workflow_result.and(end_result)
    }
}

#[derive(Debug, Clone)]
pub struct WorkflowConfig {
    pub max_loops: usize,
    pub run_learn: bool,
    /// Skip learn phase if elapsed time is below this threshold (milliseconds).
    /// Default: `300_000` (5 minutes). Set to 0 to always run learn when `run_learn` is true.
    pub learn_min_elapsed_ms: u64,
    /// Skip `check_plan` step (enabled by `--trust-the-plan`).
    pub skip_check_plan: bool,
}

pub struct Orchestrator<'a> {
    pub client: &'a mut AgentClient,
    pub prompts: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub config: WorkflowConfig,
    pub progress_callback: Box<dyn FnMut(&str) + Send + 'a>,
    pub session_dotfile_backups: SessionDotfileBackups,
}

/// Default learn-phase skip threshold (5 minutes), shared with CLI wiring.
pub const DEFAULT_LEARN_MIN_ELAPSED_MS: u64 = 300_000;

/// Returns true if learn should run given threshold and elapsed time.
/// Threshold of 0 means always run. Otherwise, run only if elapsed >= threshold.
#[must_use]
pub const fn should_run_learn_check(threshold_ms: u64, elapsed_ms: u64) -> bool {
    threshold_ms == 0 || elapsed_ms >= threshold_ms
}

impl Orchestrator<'_> {
    pub(super) fn attach_run_timing(&mut self) -> Arc<Mutex<RunTiming>> {
        self.client.attach_run_timing_for_session()
    }

    fn should_run_learn(&self) -> bool {
        let elapsed_ms = self.client.timing.as_ref().map_or(0, |t| {
            let d = t
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .elapsed_so_far();
            u64::try_from(d.as_millis()).unwrap_or(u64::MAX)
        });
        should_run_learn_check(self.config.learn_min_elapsed_ms, elapsed_ms)
    }

    pub(super) fn emit_run_timing_artifact(
        &mut self,
        timing: &Arc<Mutex<RunTiming>>,
    ) -> Result<(), WorkflowError> {
        let res = run_timing::finalize_and_emit_run_timing(&self.artifacts.run_dir, timing);
        self.client.timing = None;
        res.map_err(|e| WorkflowError(format!("run timing: {e}")))
    }

    fn fail_on_abort_result(&self) -> Result<(), WorkflowError> {
        if let Some(abort_msg) = check_abort(&self.artifacts.artifact_result_md()) {
            return Err(WorkflowError(format!("ABORT: {abort_msg}")));
        }
        Ok(())
    }

    /// Drive the full workflow.
    ///
    /// # Errors
    ///
    /// Returns [`WorkflowError`] when a prompt or review step fails.
    pub async fn run(&mut self) -> Result<(), WorkflowError> {
        let context = workflow_context_inner(self.artifacts, self.prompts, "code")
            .map_err(|e: PromptError| WorkflowError(e.0))?;
        self.run_with_pre_summary_gap(&context, mid_noop).await
    }

    /// Runs coder prompts up to the pre-summary gap, executes `mid`, then summary.
    ///
    /// # Errors
    ///
    /// Returns [`WorkflowError`] when session setup, a workflow step, `mid`, or timing emission fails.
    pub async fn run_with_pre_summary_gap(
        &mut self,
        context: &HashMap<String, String>,
        mid: PreSummaryMidFn,
    ) -> Result<(), WorkflowError> {
        let timing = self.attach_run_timing();
        let begin_res = self
            .client
            .begin_coder_session(&self.artifacts.work_dir)
            .await;
        let coder_session_began = begin_res.is_ok();
        let workflow_result = match begin_res {
            Ok(()) => {
                async {
                    run_coder_session_until_pre_summary(self, context).await?;
                    mid(self.client, self.artifacts, &self.session_dotfile_backups)
                        .await
                        .map_err(WorkflowError)?;
                    run_coder_session_summary_only(self, context).await
                }
                .await
            }
            Err(e) => Err(WorkflowError(e.0)),
        };
        let timing_result = if coder_session_began {
            self.emit_run_timing_artifact(&timing)
        } else {
            self.client.set_run_timing(None);
            Ok(())
        };
        let end_result = self
            .client
            .end_coder_session()
            .await
            .map_err(|e: AgentError| WorkflowError(e.0));
        prefer_primary_errors_over_timing(workflow_result, end_result, timing_result)
    }

    /// KPOP already finished; run regression-test then fix coder prompts, optional mid hook, summary.
    ///
    /// # Errors
    ///
    /// Returns [`WorkflowError`] when session setup, a bug phase, `mid`, or timing emission fails.
    pub async fn run_bug_remediation_gap(
        &mut self,
        context: &HashMap<String, String>,
        mid: PreSummaryMidFn,
    ) -> Result<(), WorkflowError> {
        bug_remediation::run_bug_remediation_gap(self, context, mid).await
    }
}

include!("coder_prompt_impl.rs");
