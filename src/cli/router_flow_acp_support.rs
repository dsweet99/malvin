use crate::cli::router_flow::router_flow_no_work::chat_has_malvin_done;
use crate::cli::router_flow::router_flow_prompt;
use crate::cli::router_machine::Completion;
use crate::cli::{RouterOpts, SharedOpts};
use malvin::artifacts::{RunArtifacts, SessionDotfileBackups, ensure_gate_exp_log_file};
use malvin::backends::agent_backend::SdkClient;
use malvin::prompts::PromptStore;
use malvin::run_timing::acp_post_run::RunTimingSessionEnd;
use malvin::slow_task_watch::WatchedPromptFlag;
use std::path::Path;

use super::router_flow_coder_prompts::{
    RouterInitialCoderPrompt, run_router_b_coder_prompt, run_router_followup_coder_prompt,
    run_router_initial_coder_prompt,
};
use super::router_flow_fact_agent::{FactAgentSpec, start_fact_agent_watch};

pub(crate) struct RouterAcpIterationInput<'a> {
    pub client: &'a mut SdkClient,
    pub artifacts: &'a RunArtifacts,
    pub prompt_store: &'a PromptStore,
    pub shared: &'a SharedOpts,
    pub router: &'a RouterOpts,
    pub agent_loop: usize,
    pub session_end: RunTimingSessionEnd,
}

pub(crate) async fn begin_coder_session_if_needed(
    client: &mut SdkClient,
    work_dir: &Path,
) -> Result<malvin::backends::agent_backend::CoderSessionEnsure, String> {
    client
        .start_coder_session(work_dir)
        .await
        .map_err(|e| e.to_string())
}

pub(crate) struct RouterTurnsOutcome {
    pub iteration_backups: SessionDotfileBackups,
    pub completion: Completion,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RouterExitSummarize {
    Run,
    Skip,
}

pub(crate) fn router_iteration_log_path(
    artifacts: &RunArtifacts,
    agent_loop: usize,
) -> std::path::PathBuf {
    artifacts.log_path(&format!("router_{agent_loop}"))
}

pub(crate) const fn empty_iteration_backups() -> SessionDotfileBackups {
    SessionDotfileBackups::all_missing()
}

pub(crate) fn snapshot_iteration_backups(work_dir: &Path) -> SessionDotfileBackups {
    SessionDotfileBackups::snapshot_after_ensuring_home_config(work_dir)
        .unwrap_or_else(|_| empty_iteration_backups())
}

pub(crate) async fn run_router_turns(
    input: &mut RouterAcpIterationInput<'_>,
    log_path: &Path,
) -> Result<RouterTurnsOutcome, String> {
    let model = input.shared.model.first().canonical();
    let _exp_log =
        ensure_gate_exp_log_file(input.artifacts, input.agent_loop).map_err(|e| e.to_string())?;
    input
        .client
        .force_fresh_coder_agent()
        .await
        .map_err(|e| e.to_string())?;
    let flag = WatchedPromptFlag::default();
    let _fact_watch = start_fact_agent_watch(
        &FactAgentSpec {
            store: input.prompt_store,
            artifacts: input.artifacts,
            models: &input.shared.model,
        },
        &flag,
    );
    flag.set(true);
    let initial = deliver_router_initial_turn(input, log_path).await;
    flag.set(false);
    let iteration_backups = initial?;
    let followup = finish_router_a_maybe_b(input, log_path, &model, &flag).await?;
    Ok(RouterTurnsOutcome {
        iteration_backups,
        completion: followup,
    })
}

async fn deliver_router_initial_turn(
    input: &mut RouterAcpIterationInput<'_>,
    log_path: &Path,
) -> Result<SessionDotfileBackups, String> {
    let work_dir = input.artifacts.work_dir.as_path();
    let model = input.shared.model.first().canonical();
    let include_header =
        !malvin::backends::agent_backend::session_header_is_satisfied(input.client);
    let initial = router_flow_prompt::build_router_initial_prompt(
        router_flow_prompt::RouterInitialPromptInput {
            store: input.prompt_store,
            artifacts: input.artifacts,
            model: &model,
            gates: input.router.gates,
            gates_just_ran: malvin::gate_loop_session::quality_gates_just_ran(),
            include_header,
        },
    )?;

    if include_header {
        input.client.bind_session_header_parts(
            initial.body,
            log_path.to_path_buf(),
            &initial.stdout_label,
            initial.log_who,
        );
        let begin = input.client.start_coder_session(work_dir);
        let snapshot = async {
            SessionDotfileBackups::snapshot_after_ensuring_home_config(work_dir)
                .map_err(|e| e.to_string())
        };
        let (begin_res, snapshot_res) = tokio::join!(begin, snapshot);
        begin_res.map_err(|e| e.to_string())?;
        snapshot_res
    } else {
        let iteration_backups =
            SessionDotfileBackups::snapshot_after_ensuring_home_config(work_dir)?;
        begin_coder_session_if_needed(input.client, work_dir).await?;
        run_router_initial_coder_prompt(RouterInitialCoderPrompt {
            client: input.client,
            prompt: &initial.body,
            log_path,
            stdout_bracket_label: &initial.stdout_label,
            log_who: initial.log_who,
        })
        .await?;
        Ok(iteration_backups)
    }
}

async fn finish_router_a_maybe_b(
    input: &mut RouterAcpIterationInput<'_>,
    log_path: &Path,
    model: &str,
    flag: &WatchedPromptFlag,
) -> Result<Completion, String> {
    let router_a_2 = router_flow_prompt::build_router_a_2_prompt(input.prompt_store)?;
    run_router_followup_coder_prompt(
        input.client,
        &router_a_2,
        log_path,
        malvin::prompts::router_a_2_prompt_file(),
    )
    .await?;
    if last_response_has_malvin_done(input.client) {
        return Ok(Completion::Impasse);
    }
    let router_b =
        router_flow_prompt::build_router_b_prompt(router_flow_prompt::RouterBPromptInput {
            store: input.prompt_store,
            artifacts: input.artifacts,
            model,
        })?;
    flag.set(true);
    let b_result = run_router_b_coder_prompt(input.client, &router_b, log_path).await;
    flag.set(false);
    b_result?;
    send_router_c(input, log_path).await
}

async fn send_router_c(
    input: &mut RouterAcpIterationInput<'_>,
    log_path: &Path,
) -> Result<Completion, String> {
    let router_c = router_flow_prompt::build_router_c_prompt(input.prompt_store)?;
    run_router_followup_coder_prompt(
        input.client,
        &router_c,
        log_path,
        malvin::prompts::ROUTER_C_MD,
    )
    .await?;
    Ok(if last_response_has_malvin_done(input.client) {
        Completion::Satisfied
    } else {
        Completion::StillOpen
    })
}

pub(crate) fn last_response_has_malvin_done(
    client: &malvin::backends::agent_backend::SdkClient,
) -> bool {
    client
        .last_coder_prompt_agent_response()
        .is_some_and(|chat| chat_has_malvin_done(&chat))
}

#[cfg(test)]
#[path = "router_flow_acp_support_tests.rs"]
mod router_flow_acp_support_tests;

#[cfg(test)]
#[path = "router_flow_acp_support_kiss_cov_tests.rs"]
mod router_flow_acp_support_kiss_cov_tests;

#[cfg(test)]
#[path = "router_flow_acp_support_done_tests.rs"]
mod router_flow_acp_support_done_tests;

#[cfg(test)]
#[path = "router_flow_acp_support_fresh_agent_tests.rs"]
mod router_flow_acp_support_fresh_agent_tests;
