use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::acp::{AgentClient, AgentError, ReviewerPromptPair, ReviewerRestorePolicy};
use crate::artifacts::RunArtifacts;
use crate::prompts::PromptStore;
use crate::run_timing::ReviewPairId;

use super::constants::{
    REVIEWER_FANOUT_CONCURRENCY, REVIEWER_TEMPLATE_FILE, fanout_wave_count,
};
use super::review_fanout_desc::{
    expand_review_description_line, fanout_reviewer_render_context, reviewer_output_filename,
};
use super::{WorkflowError, format_prompt_path};

const FANOUT_REVIEWER_RESTORE_POLICY: ReviewerRestorePolicy = ReviewerRestorePolicy::NoRestore;

pub struct FanoutPrepareInput<'a> {
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub context: &'a HashMap<String, String>,
    pub descriptions: &'a [String],
    pub reviewers_subdir: &'a Path,
    pub attempt: usize,
}

struct FanoutReviewerJob {
    body: String,
    who: String,
    log: PathBuf,
}

fn build_one_fanout_job(
    input: &FanoutPrepareInput<'_>,
    job_index: usize,
    line: &str,
    subdir_formatted: &str,
) -> Result<FanoutReviewerJob, WorkflowError> {
    let review_description = expand_review_description_line(line, input.context)?;
    let reviewer_filename = reviewer_output_filename(job_index);
    let job_ctx = fanout_reviewer_render_context(
        input.context,
        review_description,
        subdir_formatted,
        &reviewer_filename,
    );
    let review_body = input
        .store
        .render(REVIEWER_TEMPLATE_FILE, &job_ctx)
        .map_err(|e| WorkflowError(e.0))?;
    let review_who = format!("reviewer_{job_index:03}");
    let review_log = input
        .artifacts
        .log_path(&format!("{review_who}_attempt_{}", input.attempt));
    Ok(FanoutReviewerJob {
        body: review_body,
        who: review_who,
        log: review_log,
    })
}

fn prepare_fanout_reviewer_jobs(
    input: &FanoutPrepareInput<'_>,
) -> Result<Vec<FanoutReviewerJob>, WorkflowError> {
    let subdir_formatted = format_prompt_path(input.reviewers_subdir, &input.artifacts.work_dir);
    let mut jobs = Vec::with_capacity(input.descriptions.len());
    for (index, line) in input.descriptions.iter().enumerate() {
        jobs.push(build_one_fanout_job(
            input,
            index + 1,
            line,
            &subdir_formatted,
        )?);
    }
    Ok(jobs)
}

struct FanoutChunkEnv<'a> {
    client: &'a AgentClient,
    cwd: &'a Path,
    workspace_review_path: PathBuf,
}

async fn run_one_fanout_reviewer(
    env: &FanoutChunkEnv<'_>,
    job: &FanoutReviewerJob,
) -> Result<(), WorkflowError> {
    let pair = ReviewerPromptPair {
        cwd: env.cwd,
        workspace_review_path: &env.workspace_review_path,
        artifact_review_path: None,
        review_body: &job.body,
        review_who: &job.who,
        review_log: &job.log,
        sync_workspace_review: false,
    };
    env.client
        .run_reviewer_review(pair, ReviewPairId::Fanout, FANOUT_REVIEWER_RESTORE_POLICY)
        .await
        .map_err(|e: AgentError| WorkflowError(e.0))
}

async fn run_fanout_chunk(env: &FanoutChunkEnv<'_>, jobs: &[FanoutReviewerJob]) -> Result<(), WorkflowError> {
    if jobs.len() > REVIEWER_FANOUT_CONCURRENCY {
        return Err(WorkflowError(format!(
            "internal: fan-out chunk size {} exceeds {REVIEWER_FANOUT_CONCURRENCY}",
            jobs.len()
        )));
    }
    match jobs.len() {
        0 => Ok(()),
        1 => run_one_fanout_reviewer(env, &jobs[0]).await,
        2 => {
            tokio::try_join!(
                run_one_fanout_reviewer(env, &jobs[0]),
                run_one_fanout_reviewer(env, &jobs[1]),
            )?;
            Ok(())
        }
        3 => {
            tokio::try_join!(
                run_one_fanout_reviewer(env, &jobs[0]),
                run_one_fanout_reviewer(env, &jobs[1]),
                run_one_fanout_reviewer(env, &jobs[2]),
            )?;
            Ok(())
        }
        n => Err(WorkflowError(format!(
            "internal: unhandled fan-out chunk size {n} (extend run_fanout_chunk when raising REVIEWER_FANOUT_CONCURRENCY above 3)"
        ))),
    }
}

pub async fn run_review_fanout_jobs(
    client: &AgentClient,
    input: FanoutPrepareInput<'_>,
) -> Result<(), WorkflowError> {
    std::fs::create_dir_all(input.reviewers_subdir).map_err(|e| {
        WorkflowError(format!(
            "failed to create reviewers dir {}: {e}",
            input.reviewers_subdir.display()
        ))
    })?;
    let jobs = prepare_fanout_reviewer_jobs(&input)?;
    debug_assert_eq!(
        jobs.chunks(REVIEWER_FANOUT_CONCURRENCY).count(),
        fanout_wave_count(jobs.len())
    );
    let env = FanoutChunkEnv {
        client,
        cwd: &input.artifacts.work_dir,
        workspace_review_path: input.artifacts.workspace_review_md(),
    };
    for chunk in jobs.chunks(REVIEWER_FANOUT_CONCURRENCY) {
        run_fanout_chunk(&env, chunk).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::constants::REVIEWER_FANOUT_CONCURRENCY;
    use super::super::review_fanout_desc::load_review_description_lines;
    use super::*;
    use crate::artifacts::RunArtifacts;
    use crate::prompts::PromptStore;

    #[test]
    fn kiss_stringify_review_fanout_run_units() {
        let _ = stringify!(super::run_review_fanout_jobs);
        let _ = stringify!(super::FanoutPrepareInput);
    }

    #[test]
    fn prepare_fanout_builds_one_job_per_description_line() {
        let tmp = tempfile::tempdir().unwrap();
        let work = tmp.path().join("ws");
        std::fs::create_dir_all(&work).unwrap();
        let run_dir = tmp.path().join("_malvin").join("run1");
        std::fs::create_dir_all(&run_dir).unwrap();
        let plan_path = work.join("plan.md");
        std::fs::write(&plan_path, "# plan\n").unwrap();
        let artifacts = RunArtifacts {
            run_dir: run_dir.clone(),
            plan_path,
            work_dir: work,
        };
        let store = PromptStore::default_store();
        let descriptions = load_review_description_lines(&store).expect("descriptions");
        let reviewers_subdir = run_dir.join("reviewers_attempt_1");
        let context =
            super::super::workflow_context(&artifacts, &store, "code").expect("workflow_context");
        let input = FanoutPrepareInput {
            store: &store,
            artifacts: &artifacts,
            context: &context,
            descriptions: &descriptions,
            reviewers_subdir: &reviewers_subdir,
            attempt: 1,
        };
        let jobs = prepare_fanout_reviewer_jobs(&input).expect("prepare jobs");
        assert_eq!(jobs.len(), descriptions.len());
        assert_eq!(jobs.len(), super::super::review_fanout_desc::embedded_review_description_job_count());
        let body = &jobs[0].body;
        assert!(
            body.contains("Write your executive summary and tl;dr to"),
            "reviewer prompt must include output instruction: {body}"
        );
        assert!(
            body.contains("reviewer_001.md"),
            "reviewer prompt must name output file: {body}"
        );
        assert!(
            !body.contains("reviewer_001.md."),
            "output path must not include a trailing period after the filename: {body}"
        );
        let line = body
            .lines()
            .find(|l| l.starts_with("Write your executive summary and tl;dr to"))
            .expect("reviewer output path line");
        assert!(
            line.contains("reviewers_attempt_1/reviewer_001.md"),
            "reviewer output path line must name attempt dir and file: {line}"
        );
        assert!(
            !line.trim_end().ends_with("reviewer_001.md."),
            "output path must not include a trailing period after the filename: {line}"
        );
    }

    #[test]
    fn fanout_reviewer_restore_policy_avoids_concurrent_dotfile_races() {
        const { assert!(REVIEWER_FANOUT_CONCURRENCY > 1) };
        assert_eq!(
            super::FANOUT_REVIEWER_RESTORE_POLICY,
            ReviewerRestorePolicy::NoRestore,
            "parallel reviewers must not snapshot/restore shared workspace dotfiles"
        );
    }

}
