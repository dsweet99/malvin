//! Implement → review loops.
//!
//! Helper-focused unit tests live in [`crate::orchestrator_tests`] (crate root) so `kiss` can
//! attribute coverage consistently; see `.kissignore`.

use crate::acp::{AgentClient, AgentError};
use crate::artifacts::RunArtifacts;
use crate::prompts::PromptStore;
use crate::run_timing::{self, RunTiming, TimingPhase};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

include!("helpers.rs");

pub(crate) mod review_context;
mod review_loop;

use review_context::ReviewPhaseArgs;

use workflow_context as workflow_context_inner;

/// Workflow stopped after `max_loops` without LGTM.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct WorkflowError(pub String);

/// Prefer workflow or session-teardown errors over run-timing artifact errors.
pub(crate) fn prefer_primary_errors_over_timing(
    workflow_result: Result<(), WorkflowError>,
    end_result: Result<(), WorkflowError>,
    timing_result: Result<(), WorkflowError>,
) -> Result<(), WorkflowError> {
    match (workflow_result, end_result) {
        (Ok(()), Ok(())) => timing_result,
        (wf, er) => {
            let _ = timing_result;
            match (wf, er) {
                (Err(e), _) | (Ok(()), Err(e)) => Err(e),
                (Ok(()), Ok(())) => unreachable!("outer match excludes both Ok"),
            }
        }
    }
}

/// Review loop configuration.
#[derive(Debug, Clone)]
pub struct WorkflowConfig {
    pub max_loops: usize,
    pub run_learn: bool,
}

/// Runs implement + two review phases + optional learn pass.
pub struct Orchestrator<'a> {
    pub client: &'a mut AgentClient,
    pub prompts: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub config: WorkflowConfig,
    pub progress_callback: Box<dyn FnMut(&str) + Send + 'a>,
}

impl Orchestrator<'_> {
    fn attach_run_timing(&mut self) -> Arc<Mutex<RunTiming>> {
        self.client.attach_run_timing_for_session()
    }

    fn emit_run_timing_artifact(
        &mut self,
        timing: &Arc<Mutex<RunTiming>>,
    ) -> Result<(), WorkflowError> {
        let res = run_timing::finalize_and_emit_run_timing(&self.artifacts.run_dir, timing);
        self.client.timing = None;
        res.map_err(|e| WorkflowError(format!("run timing: {e}")))
    }

    /// Drive the full workflow.
    ///
    /// # Errors
    ///
    /// Returns [`WorkflowError`] when a prompt or review step fails.
    pub async fn run(&mut self) -> Result<(), WorkflowError> {
        let context = workflow_context_inner(self.artifacts);
        let timing = self.attach_run_timing();
        let begin_res = self
            .client
            .begin_coder_session(&self.artifacts.work_dir)
            .await;
        let workflow_result = match begin_res {
            Ok(()) => self.run_with_coder_session(&context).await,
            Err(e) => Err(WorkflowError(e.0)),
        };
        let timing_result = self.emit_run_timing_artifact(&timing);
        let end_result = self
            .client
            .end_coder_session()
            .await
            .map_err(|e: AgentError| WorkflowError(e.0));
        prefer_primary_errors_over_timing(workflow_result, end_result, timing_result)
    }

    async fn run_with_coder_session(
        &mut self,
        context: &HashMap<String, String>,
    ) -> Result<(), WorkflowError> {
        (self.progress_callback)("Implement");
        self.run_coder_prompt("implement.md", context, "main", TimingPhase::Implement)
            .await?;

        self.run_review_phase(ReviewPhaseArgs {
            review_prompt: "review_1.md",
            progress_label: "Review-1",
            phase_id: "review_1",
            context,
        })
        .await?;
        self.run_review_phase(ReviewPhaseArgs {
            review_prompt: "review_2.md",
            progress_label: "Review-2",
            phase_id: "review_2",
            context,
        })
        .await?;

        if self.config.run_learn {
            (self.progress_callback)("Learn");
            self.run_coder_prompt("learn.md", context, "final", TimingPhase::Learn)
                .await?;
        }
        Ok(())
    }

    pub(super) async fn run_coder_prompt(
        &mut self,
        filename: &str,
        context: &HashMap<String, String>,
        suffix: &str,
        llm_phase: TimingPhase,
    ) -> Result<(), WorkflowError> {
        let prompt = self
            .prompts
            .render(filename, context)
            .map_err(|e| WorkflowError(e.0))?;
        let stem = prompt_md_stem(filename);
        let log = self.artifacts.log_path(&format!("coder_{stem}_{suffix}"));
        self.client
            .run_coder_prompt(&prompt, &log, stem, Some(llm_phase))
            .await
            .map_err(|e: AgentError| WorkflowError(e.0))?;
        Ok(())
    }
}
