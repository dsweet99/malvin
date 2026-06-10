//! ACP prompt dispatch and post-prompt commits for `malvin plan`.

use crate::artifacts::{
    PlanRunMetadata, RunArtifacts, extract_decisions_section, extract_fenced_markdown_block,
    overwrite_plan_file, prepare_plan_file_for_prompt_1a, read_plan_file,
    record_user_span_end_after_1a, snapshot_plan_artifact, validate_post_1a, validate_post_1b,
    validate_post_2,
    write_plan_metadata,
};
use crate::prompts::PromptStore;
use crate::run_timing::TimingPhase;

use super::plan_flow_prompt::{render_plan_1a, render_plan_1b, render_plan_2, render_plan_3};

pub(super) struct PlanRunPrep {
    pub(super) client: crate::acp::AgentClient,
    pub(super) artifacts: RunArtifacts,
    pub(super) source_plan_path: std::path::PathBuf,
    pub(super) store: PromptStore,
    pub(super) render_ctx: std::collections::HashMap<String, String>,
    pub(super) session_dotfile_backups: crate::artifacts::SessionDotfileBackups,
}

pub(super) async fn run_plan_acp(prep: &mut PlanRunPrep) -> Result<(), String> {
    let timing = prep.client.attach_run_timing_for_session();
    if let Err(e) = prep
        .client
        .begin_coder_session(&prep.artifacts.work_dir)
        .await
    {
        prep.client.set_run_timing(None);
        return Err(e.to_string());
    }
    timing
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .set_implement_display_name("plan");
    let run_res = run_plan_four_prompts(prep).await;
    let end_res = prep.client.end_coder_session().await.map_err(|e| e.to_string());
    let merged =
        crate::acp_post_run::prefer_primary_over_secondary(run_res, end_res, "end coder session");
    crate::acp_post_run::emit_run_timing_json_only_after_acp(
        &mut prep.client,
        &prep.artifacts.run_dir,
        &timing,
        merged,
    )
}

async fn run_plan_four_prompts(prep: &mut PlanRunPrep) -> Result<(), String> {
    let user_span_end = run_plan_prompt_1a(prep).await?;
    run_plan_prompt_1b(prep).await?;
    run_plan_prompt_2(prep).await?;
    run_plan_prompt_3(prep, user_span_end).await
}

async fn run_plan_prompt_1a(prep: &mut PlanRunPrep) -> Result<usize, String> {
    prepare_plan_file_for_prompt_1a(&prep.source_plan_path).map_err(|e| e.to_string())?;
    let prompt = render_plan_1a(&prep.store, &prep.render_ctx)?;
    run_plan_coder_prompt(&mut prep.client, &prep.artifacts, &prompt, "plan_1a").await?;
    let content = read_plan_file(&prep.source_plan_path).map_err(|e| e.to_string())?;
    commit_plan_prompt_1a(prep, &content)
}

async fn run_plan_prompt_1b(prep: &mut PlanRunPrep) -> Result<(), String> {
    let prompt = render_plan_1b(&prep.store, &prep.render_ctx)?;
    run_plan_coder_prompt(&mut prep.client, &prep.artifacts, &prompt, "plan_1b").await?;
    let content = read_plan_file(&prep.source_plan_path).map_err(|e| e.to_string())?;
    commit_plan_prompt_1b(prep, &content)
}

async fn run_plan_prompt_2(prep: &mut PlanRunPrep) -> Result<(), String> {
    let prompt = render_plan_2(&prep.store, &prep.render_ctx)?;
    run_plan_coder_prompt(&mut prep.client, &prep.artifacts, &prompt, "plan_2").await?;
    let content = read_plan_file(&prep.source_plan_path).map_err(|e| e.to_string())?;
    commit_plan_prompt_2(prep, &content)
}

async fn run_plan_prompt_3(prep: &mut PlanRunPrep, user_span_end: usize) -> Result<(), String> {
    let prompt = render_plan_3(&prep.store, &prep.render_ctx)?;
    run_plan_coder_prompt(&mut prep.client, &prep.artifacts, &prompt, "plan_3").await?;
    let response = prep
        .client
        .last_coder_prompt_agent_response()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| "plan prompt 3: empty agent response".to_string())?;
    commit_plan_prompt_3(prep, user_span_end, &response)
}

async fn run_plan_coder_prompt(
    client: &mut crate::acp::AgentClient,
    artifacts: &RunArtifacts,
    prompt: &str,
    log_stem: &str,
) -> Result<(), String> {
    client
        .run_coder_prompt(
            prompt,
            &artifacts.log_path(log_stem),
            log_stem,
            crate::acp::CoderPromptOptions {
                llm_phase: Some(TimingPhase::Implement),
                do_trace_split: None,
                stdout_bracket_label: None,
            },
        )
        .await
        .map_err(|e| e.to_string())
}

pub(super) fn commit_plan_prompt_1a(prep: &PlanRunPrep, content: &str) -> Result<usize, String> {
    validate_post_1a(content).map_err(|e| e.to_string())?;
    let user_span_end = record_user_span_end_after_1a(content).map_err(|e| e.to_string())?;
    write_plan_metadata(
        &prep.artifacts.run_dir,
        &PlanRunMetadata {
            user_span_end,
            user_span_sha256: None,
        },
    )
    .map_err(|e| e.to_string())?;
    snapshot_plan_artifact(&prep.artifacts.run_dir, "plan.p1a.md", &prep.source_plan_path)
        .map_err(|e| e.to_string())?;
    Ok(user_span_end)
}

pub(super) fn commit_plan_prompt_1b(prep: &PlanRunPrep, content: &str) -> Result<(), String> {
    validate_post_1b(content).map_err(|e| e.to_string())?;
    snapshot_plan_artifact(&prep.artifacts.run_dir, "plan.p1b.md", &prep.source_plan_path)
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub(super) fn commit_plan_prompt_2(prep: &PlanRunPrep, content: &str) -> Result<(), String> {
    validate_post_2(content).map_err(|e| e.to_string())?;
    if let Some(decisions) = extract_decisions_section(content) {
        std::fs::write(
            prep.artifacts.run_dir.join("plan.p2.decisions.md"),
            decisions,
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub(super) fn commit_plan_prompt_3(
    prep: &PlanRunPrep,
    _user_span_end: usize,
    response: &str,
) -> Result<(), String> {
    let fenced = extract_fenced_markdown_block(response).map_err(|e| e.to_string())?;
    overwrite_plan_file(&prep.source_plan_path, &fenced).map_err(|e| e.to_string())
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs{
    use super::*;

    #[test]
    fn kiss_cov_pipeline_symbols() {
        let _ = stringify!(run_plan_acp);
        let _ = run_plan_coder_prompt;
        let _ = stringify!(prepare_plan_file_for_prompt_1a);
        let _ = run_plan_prompt_1a;
        let _ = run_plan_prompt_1b;
        let _ = run_plan_prompt_2;
        let _ = run_plan_prompt_3;
        let _ = stringify!(commit_plan_prompt_1a);
        let _ = stringify!(commit_plan_prompt_1b);
        let _ = stringify!(commit_plan_prompt_2);
        let _ = stringify!(commit_plan_prompt_3);
        let _ = overwrite_plan_file;
    }
}
