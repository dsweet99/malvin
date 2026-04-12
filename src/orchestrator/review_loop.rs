//! Review phase loop: reviewer + kpop pair, concerns.

use crate::acp::{AgentError, ReviewerPromptPair};
use crate::review_sync::{is_lgtm, sync_review_file};
use crate::run_timing::{ReviewPairId, TimingPhase};

use super::Orchestrator;
use super::WorkflowError;
use super::clear_review_file;
use super::prompt_md_stem;
use super::review_context::{ReviewAttemptCtx, ReviewPhaseArgs};

impl Orchestrator<'_> {
    pub(super) async fn run_review_phase(
        &mut self,
        phase: ReviewPhaseArgs<'_>,
    ) -> Result<(), WorkflowError> {
        let review_path = self.artifacts.run_dir.join("review.md");
        let workspace_review_path = self.artifacts.work_dir.join("review.md");

        for attempt in 1..=self.config.max_loops {
            let ctx = ReviewAttemptCtx {
                review_prompt: phase.review_prompt,
                progress_label: phase.progress_label,
                phase_id: phase.phase_id,
                attempt,
                workspace_review_path: &workspace_review_path,
                review_path: &review_path,
                context: phase.context,
            };
            if self.review_phase_single_attempt(ctx).await? {
                return Ok(());
            }
        }
        Err(WorkflowError(format!(
            "Did not receive LGTM for {} within max loops.",
            phase.review_prompt
        )))
    }

    async fn review_phase_single_attempt(
        &mut self,
        ctx: ReviewAttemptCtx<'_>,
    ) -> Result<bool, WorkflowError> {
        (self.progress_callback)(&format!("{} (attempt {})", ctx.progress_label, ctx.attempt));

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

        let pair_id = match ctx.phase_id {
            "review_2" => ReviewPairId::Two,
            _ => ReviewPairId::One,
        };
        self.run_reviewer_pair_for_attempt(&ctx, &review_body, &kpop_body, pair_id)
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
            TimingPhase::Concerns,
        )
        .await?;
        Ok(false)
    }

    async fn run_reviewer_pair_for_attempt(
        &mut self,
        ctx: &ReviewAttemptCtx<'_>,
        review_body: &str,
        kpop_body: &str,
        pair_id: ReviewPairId,
    ) -> Result<(), WorkflowError> {
        let stem = prompt_md_stem(ctx.review_prompt);
        let review_log = self
            .artifacts
            .log_path(&format!("reviewer_{stem}_attempt_{}", ctx.attempt));
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
            .run_reviewer_review_and_kpop(pair, pair_id)
            .await
            .map_err(|e: AgentError| WorkflowError(e.0))?;
        Ok(())
    }
}
