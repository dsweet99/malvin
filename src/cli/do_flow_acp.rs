//! ACP session helpers for `--do`.

use crate::agent_backend::{
    agent_backend_attach_run_timing_for_session, agent_backend_set_implement_display_name,
    agent_backend_set_run_timing, AgentBackend,
};
use crate::artifacts::RunArtifacts;
use crate::run_timing::TimingPhase;

use super::do_flow_prompt;

pub(super) async fn run_do_coder_prompt(
    client: &mut AgentBackend,
    artifacts: &RunArtifacts,
    coder: &do_flow_prompt::DoCoderRun,
) -> Result<(), String> {
    let (ref header, ref user) = coder.header_user_for_trace;
    client
        .run_coder_prompt(
            &coder.combined,
            &artifacts.log_path("do"),
            "do",
            crate::acp::CoderPromptOptions {
                llm_phase: Some(TimingPhase::Implement),
                do_trace_split: Some((header.as_str(), user.as_str())),
                stdout_bracket_label: None,
                ..Default::default()
            },
        )
        .await
        .map_err(|e| e.to_string())
}

pub(super) async fn run_do_acp(
    client: &mut AgentBackend,
    artifacts: &RunArtifacts,
    coder: do_flow_prompt::DoCoderRun,
) -> Result<(), String> {
    let timing = agent_backend_attach_run_timing_for_session(client);
    if !client.has_open_coder_session() {
        if let Err(e) = client.begin_coder_session(&artifacts.work_dir).await {
            agent_backend_set_run_timing(client, None);
            return Err(e.to_string());
        }
    }
    agent_backend_set_implement_display_name(client, "do");
    let run_res = run_do_coder_prompt(client, artifacts, &coder).await;
    let end_res = client.end_coder_session().await.map_err(|e| e.to_string());
    let merged =
        crate::acp_post_run::prefer_primary_over_secondary(run_res, end_res, "end coder session");
    crate::acp_post_run::emit_run_timing_json_only_after_backend(
        client,
        &artifacts.run_dir,
        &timing,
        merged,
    )
}
