//! Implement → review loops.
//!
//! Helper-focused unit tests live in [`crate::orchestrator_tests`] (crate root) so `kiss` can
//! attribute coverage consistently; see `.kissignore`.

use std::collections::HashMap;
use std::path::Path;

use crate::acp::{AgentClient, AgentError};
use crate::artifacts::RunArtifacts;
use crate::edit_efficiency::{EditEfficiencyMeter, finish_and_write_report, maybe_checkpoint};
use crate::prompts::PromptStore;
use tracing::debug;

include!("helpers.rs");

pub(crate) mod review_context;
mod review_loop;

use review_context::ReviewPhaseArgs;

use workflow_context as workflow_context_inner;

/// Workflow stopped after `max_loops` without LGTM.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct WorkflowError(pub String);

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
    /// Drive the full workflow.
    ///
    /// # Errors
    ///
    /// Returns [`WorkflowError`] when a prompt or review step fails.
    pub async fn run(&mut self) -> Result<(), WorkflowError> {
        let context = workflow_context_inner(self.artifacts);

        self.client
            .begin_coder_session(&self.artifacts.work_dir)
            .await
            .map_err(|e: AgentError| WorkflowError(e.0))?;

        let mut edit_efficiency = match EditEfficiencyMeter::new(&self.artifacts.work_dir) {
            Ok(m) => Some(m),
            Err(e) => {
                debug!(target: "malvin::edit_efficiency", ?e, "skipping edit efficiency (not a git repo or snapshot failed)");
                None
            }
        };

        let workflow_result = self
            .run_with_coder_session(&context, &mut edit_efficiency)
            .await;

        finish_and_write_report(edit_efficiency, &self.artifacts.run_dir);

        let end_result = self
            .client
            .end_coder_session()
            .await
            .map_err(|e: AgentError| WorkflowError(e.0));

        match (workflow_result, end_result) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(e), _) | (Ok(()), Err(e)) => Err(e),
        }
    }

    async fn run_with_coder_session(
        &mut self,
        context: &HashMap<String, String>,
        edit_efficiency: &mut Option<EditEfficiencyMeter>,
    ) -> Result<(), WorkflowError> {
        (self.progress_callback)("Implement");
        self.run_coder_prompt("implement.md", context, "main", edit_efficiency)
            .await?;

        self.run_review_phase(
            ReviewPhaseArgs {
                review_prompt: "review_1.md",
                progress_label: "Review-1",
                phase_id: "review_1",
                context,
            },
            edit_efficiency,
        )
            .await?;
        self.run_review_phase(
            ReviewPhaseArgs {
                review_prompt: "review_2.md",
                progress_label: "Review-2",
                phase_id: "review_2",
                context,
            },
            edit_efficiency,
        )
            .await?;

        if self.config.run_learn {
            (self.progress_callback)("Learn");
            self.run_coder_prompt("learn.md", context, "final", edit_efficiency)
                .await?;
        }
        Ok(())
    }

    pub(super) async fn run_coder_prompt(
        &mut self,
        filename: &str,
        context: &HashMap<String, String>,
        suffix: &str,
        edit_efficiency: &mut Option<EditEfficiencyMeter>,
    ) -> Result<(), WorkflowError> {
        let prompt = self
            .prompts
            .render(filename, context)
            .map_err(|e| WorkflowError(e.0))?;
        let stem = prompt_md_stem(filename);
        let log = self
            .artifacts
            .log_path(&format!("coder_{stem}_{suffix}"));
        self.client
            .run_coder_prompt(&prompt, &log)
            .await
            .map_err(|e: AgentError| WorkflowError(e.0))?;
        maybe_checkpoint(edit_efficiency);
        Ok(())
    }
}

