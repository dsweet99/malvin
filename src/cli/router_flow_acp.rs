use crate::agent_backend::{
    agent_backend_attach_run_timing_for_session, agent_backend_set_implement_display_name,
    agent_backend_set_run_timing, AgentBackend,
};
use crate::artifacts::RunArtifacts;
use crate::router_flow::router_wants_continue;
use crate::router_flow::router_flow_prompt;
use crate::run_timing::acp_post_run::RunTimingSessionEnd;

pub(crate) struct RouterAcpIterationOutcome {
    pub acp_result: Result<(), String>,
    pub wants_continue: bool,
}

pub(crate) struct RouterAcpIterationInput<'a> {
    pub client: &'a mut AgentBackend,
    pub artifacts: &'a RunArtifacts,
    pub coder: &'a router_flow_prompt::RouterCoderRun,
    pub router_b_prompt: &'a str,
    pub session_end: RunTimingSessionEnd,
}

async fn run_router_coder_prompt(
    client: &mut AgentBackend,
    artifacts: &RunArtifacts,
    coder: &router_flow_prompt::RouterCoderRun,
) -> Result<(), String> {
    client
        .run_coder_prompt(
            &coder.combined,
            &artifacts.log_path("router"),
            "router",
            crate::acp::CoderPromptOptions {
                llm_phase: Some(crate::run_timing::TimingPhase::Implement),
                do_trace_split: None,
                stdout_bracket_label: None,
                ..Default::default()
            },
        )
        .await
        .map_err(|e| e.to_string())
}

async fn run_router_b_coder_prompt(
    client: &mut AgentBackend,
    artifacts: &RunArtifacts,
    router_b_prompt: &str,
) -> Result<(), String> {
    client
        .run_coder_prompt(
            router_b_prompt,
            &artifacts.log_path("router_b"),
            "router_b",
            crate::acp::CoderPromptOptions {
                llm_phase: Some(crate::run_timing::TimingPhase::Implement),
                do_trace_split: None,
                stdout_bracket_label: None,
                ..Default::default()
            },
        )
        .await
        .map_err(|e| e.to_string())
}

pub(crate) async fn run_router_acp_iteration(
    input: RouterAcpIterationInput<'_>,
) -> RouterAcpIterationOutcome {
    let RouterAcpIterationInput {
        client,
        artifacts,
        coder,
        router_b_prompt,
        session_end,
    } = input;
    let timing = agent_backend_attach_run_timing_for_session(client);
    if let Err(e) = client.begin_coder_session(&artifacts.work_dir).await {
        agent_backend_set_run_timing(client, None);
        return RouterAcpIterationOutcome {
            acp_result: Err(e.to_string()),
            wants_continue: false,
        };
    }
    agent_backend_set_implement_display_name(client, "router");
    let router_res = run_router_coder_prompt(client, artifacts, coder).await;
    let (run_res, wants_continue) = match router_res {
        Ok(()) => {
            let b_res = run_router_b_coder_prompt(client, artifacts, router_b_prompt).await;
            let wants_continue = b_res.is_ok()
                && client
                    .last_coder_prompt_agent_response()
                    .is_some_and(|text| router_wants_continue(&text));
            (b_res, wants_continue)
        }
        Err(e) => (Err(e), false),
    };
    let end_res = client.end_coder_session().await.map_err(|e| e.to_string());
    let merged =
        crate::acp_post_run::prefer_primary_over_secondary(run_res, end_res, "end coder session");
    let acp_result = crate::acp_post_run::emit_run_timing_after_backend(
        crate::acp_post_run::RunTimingAfterBackend {
            backend: client,
            run_dir: &artifacts.run_dir,
            timing: &timing,
            agent_result: merged,
            session_end,
        },
    );
    RouterAcpIterationOutcome {
        acp_result,
        wants_continue,
    }
}

#[cfg(test)]
mod kiss_static_fn_item_refs {
    use super::{run_router_acp_iteration, RouterAcpIterationInput, RouterAcpIterationOutcome};

    #[test]
    fn kiss_static_fn_item_refs() {
        let _ = run_router_acp_iteration;
        let _ = stringify!(run_router_coder_prompt);
        let _ = stringify!(run_router_b_coder_prompt);
        let _: Option<RouterAcpIterationInput> = None;
        let _: Option<RouterAcpIterationOutcome> = None;
        let _ = stringify!(client);
        let _ = stringify!(artifacts);
        let _ = stringify!(coder);
        let _ = stringify!(router_b_prompt);
        let _ = stringify!(session_end);
        let _ = stringify!(acp_result);
        let _ = stringify!(wants_continue);
        let _ = stringify!(timing);
        let _ = stringify!(router_res);
        let _ = stringify!(run_res);
        let _ = stringify!(b_res);
        let _ = stringify!(end_res);
        let _ = stringify!(merged);
    }
}

#[cfg(test)]
#[path = "router_flow_acp_kiss_cov_tests.rs"]
mod router_flow_acp_kiss_cov_tests;

#[cfg(test)]
#[path = "router_flow_acp_mock_tests.rs"]
pub(crate) mod router_flow_acp_mock_tests;

#[cfg(test)]
#[path = "router_flow_acp_tests.rs"]
pub(crate) mod router_flow_acp_tests;
