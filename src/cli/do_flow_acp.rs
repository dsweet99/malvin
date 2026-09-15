use malvin::agent_backend::SdkClient;
use malvin::artifacts::RunArtifacts;
use crate::cli::one_shot_session::OneShotCoderGuard;
use malvin::run_timing::TimingPhase;

use super::do_flow_prompt;

pub(super) async fn run_do_coder_prompt(
    client: &mut SdkClient,
    artifacts: &RunArtifacts,
    coder: &do_flow_prompt::DoCoderRun,
) -> Result<(), String> {
    let (ref header, ref user) = coder.header_user_for_trace;
    client
        .active_coder_session().map_err(|e| e.to_string())?.run_coder_prompt(
            &coder.combined,
            &artifacts.log_path("do"),
            "do",
            malvin::acp::CoderPromptOptions {
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
    client: &mut SdkClient,
    artifacts: &RunArtifacts,
    coder: do_flow_prompt::DoCoderRun,
) -> Result<(), String> {
    let guard = OneShotCoderGuard::begin(client, artifacts, "do").await?;
    let run_res = run_do_coder_prompt(client, artifacts, &coder).await;
    guard.finish(client, run_res).await
}
