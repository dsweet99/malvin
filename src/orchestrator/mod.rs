//! Implement → review loops.
//!
//! Helper-focused unit tests live in [`crate::orchestrator_tests`] (crate root) so `kiss` can
//! attribute coverage consistently; see `.kissignore`.

use std::collections::HashMap;
use std::path::Path;

use crate::acp::{AgentClient, AgentError, ReviewerPromptPair};
use crate::artifacts::RunArtifacts;
use crate::prompts::PromptStore;
use crate::review_sync::{is_lgtm, sync_review_file};

include!("helpers.rs");

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

struct ReviewAttemptCtx<'a> {
    review_prompt: &'a str,
    progress_label: &'a str,
    phase_id: &'a str,
    attempt: usize,
    workspace_review_path: &'a Path,
    review_path: &'a Path,
    context: &'a HashMap<String, String>,
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

        let workflow_result = self.run_with_coder_session(&context).await;

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
    ) -> Result<(), WorkflowError> {
        (self.progress_callback)("Implement");
        self.run_coder_prompt("implement.md", context, "main").await?;

        self.run_review_phase("review_1.md", "Review-1", "review_1", context)
            .await?;
        self.run_review_phase("review_2.md", "Review-2", "review_2", context)
            .await?;

        if self.config.run_learn {
            (self.progress_callback)("Learn");
            self.run_coder_prompt("learn.md", context, "final").await?;
        }
        Ok(())
    }

    async fn run_review_phase(
        &mut self,
        review_prompt: &str,
        progress_label: &str,
        phase_id: &str,
        context: &HashMap<String, String>,
    ) -> Result<(), WorkflowError> {
        let review_path = self.artifacts.run_dir.join("review.md");
        let workspace_review_path = self.artifacts.work_dir.join("review.md");

        for attempt in 1..=self.config.max_loops {
            let ctx = ReviewAttemptCtx {
                review_prompt,
                progress_label,
                phase_id,
                attempt,
                workspace_review_path: &workspace_review_path,
                review_path: &review_path,
                context,
            };
            if self.review_phase_single_attempt(ctx).await? {
                return Ok(());
            }
        }
        Err(WorkflowError(format!(
            "Did not receive LGTM for {review_prompt} within max loops."
        )))
    }

    async fn review_phase_single_attempt(
        &mut self,
        ctx: ReviewAttemptCtx<'_>,
    ) -> Result<bool, WorkflowError> {
        (self.progress_callback)(&format!(
            "{} (attempt {})",
            ctx.progress_label, ctx.attempt
        ));

        if ctx.review_prompt.starts_with("review_") {
            clear_review_file(&self.artifacts.run_dir.join("review.md"));
            clear_review_file(&self.artifacts.work_dir.join("review.md"));
        }

        let review_body = self
            .prompts
            .render(ctx.review_prompt, ctx.context)
            .map_err(|e| WorkflowError(e.0))?;
        let kpop_body = self
            .prompts
            .render("kpop.md", ctx.context)
            .map_err(|e| WorkflowError(e.0))?;

        self.run_reviewer_pair_for_attempt(&ctx, &review_body, &kpop_body)
            .await?;

        sync_review_file(ctx.workspace_review_path, ctx.review_path);
        if is_lgtm(ctx.review_path) {
            return Ok(true);
        }
        (self.progress_callback)(&format!("Concerns (attempt {})", ctx.attempt));
        self.run_coder_prompt(
            "concerns.md",
            ctx.context,
            &format!("{}_attempt_{}", ctx.phase_id, ctx.attempt),
        )
        .await?;
        Ok(false)
    }

    async fn run_reviewer_pair_for_attempt(
        &mut self,
        ctx: &ReviewAttemptCtx<'_>,
        review_body: &str,
        kpop_body: &str,
    ) -> Result<(), WorkflowError> {
        let stem = prompt_md_stem(ctx.review_prompt);
        let review_log = self.artifacts.log_path(&format!(
            "reviewer_{stem}_attempt_{}",
            ctx.attempt
        ));
        let kpop_log = self.artifacts.log_path(&format!(
            "reviewer_kpop_{}_attempt_{}",
            ctx.phase_id, ctx.attempt
        ));

        let pair = ReviewerPromptPair {
            cwd: &self.artifacts.work_dir,
            workspace_review_path: ctx.workspace_review_path,
            artifact_review_path: ctx.review_path,
            review_body,
            kpop_body,
            review_log: &review_log,
            kpop_log: &kpop_log,
        };
        self.client
            .run_reviewer_review_and_kpop(pair)
            .await
            .map_err(|e: AgentError| WorkflowError(e.0))
    }

    async fn run_coder_prompt(
        &mut self,
        filename: &str,
        context: &HashMap<String, String>,
        suffix: &str,
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
            .map_err(|e: AgentError| WorkflowError(e.0))
    }
}

