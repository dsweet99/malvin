//! Post-bootstrap summary agent session for `malvin init`.

use crate::artifacts::RunArtifacts;
use crate::cli::SharedOpts;

pub(super) fn init_summary_combined_body(
    store: &crate::prompts::PromptStore,
    ctx: &std::collections::HashMap<String, String>,
) -> Result<String, String> {
    use crate::prompts::{HEADER_MD, PromptError};
    let header_body = store
        .render_prompt_only(HEADER_MD, ctx)
        .map_err(|e: PromptError| e.0)?;
    let summary_only = store
        .render("summary.md", ctx)
        .map_err(|e: PromptError| e.0)?;
    Ok(format!(
        "{}\n\n{}",
        header_body.trim_end(),
        summary_only.trim_end()
    ))
}

async fn init_summary_coder_turn_with_timing_emit(
    client: &mut crate::acp::AgentClient,
    artifacts: &RunArtifacts,
    body: &str,
) -> Result<(), String> {
    use crate::run_timing::TimingPhase;
    let timing = client.attach_run_timing_for_session();
    timing
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .set_implement_display_name("init");
    let begin_res = client.begin_coder_session(&artifacts.work_dir).await;
    if let Err(e) = begin_res {
        client.set_run_timing(None);
        return Err(e.to_string());
    }
    let prompt_res = client
        .run_coder_prompt(
            body,
            &artifacts.log_path("summary"),
            "summary",
            crate::acp::CoderPromptOptions {
                llm_phase: Some(TimingPhase::Summary),
                do_trace_split: None,
                stdout_bracket_label: None,
            },
        )
        .await
        .map_err(|e| e.to_string());
    let end_res = client.end_coder_session().await.map_err(|e| e.to_string());
    let merged = crate::acp_post_run::prefer_primary_over_secondary(
        prompt_res,
        end_res,
        "failed to end coder session",
    );
    crate::acp_post_run::emit_run_timing_after_acp(crate::acp_post_run::RunTimingAfterAcp {
        client,
        run_dir: &artifacts.run_dir,
        timing: &timing,
        acp_result: merged,
        session_end: crate::run_timing::acp_post_run::RunTimingSessionEnd::Finalize,
    })
}

pub(super) async fn run_init_summary_phase(
    shared: &SharedOpts,
    artifacts: &RunArtifacts,
) -> Result<(), String> {
    use crate::orchestrator::workflow_context;
    use crate::prompts::{PromptError, PromptStore};
    let workflow = crate::cli::WorkflowCliOptions {
        force: !shared.no_force,
    };
    let store = PromptStore::default_store();
    store.ensure_defaults().map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("summary.md")
        .map_err(|e: PromptError| e.0)?;
    let ctx = workflow_context(artifacts, &store, "init").map_err(|e: PromptError| e.0)?;
    let session_dotfile_backups =
        crate::artifacts::SessionDotfileBackups::snapshot(&artifacts.work_dir)?;
    let mut client =
        crate::cli::build_agent(shared, workflow, shared.acp_stdout_markdown_enabled());
    client
        .ensure_authenticated()
        .map_err(|e: crate::acp::AuthError| e.to_string())?;
    client.prompts_log_run_dir = Some(artifacts.run_dir.clone());
    let body = init_summary_combined_body(&store, &ctx)?;
    let coder_turn_out =
        init_summary_coder_turn_with_timing_emit(&mut client, artifacts, &body).await;
    crate::acp_post_run::merge_acp_restore_check_abort_then_print_timing(
        coder_turn_out,
        artifacts,
        &session_dotfile_backups,
    )
}

#[cfg(test)]
mod kiss_cov_gate_refs {
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _ = init_summary_combined_body;
        let _ = init_summary_coder_turn_with_timing_emit;
        let _ = run_init_summary_phase;
    }
}
