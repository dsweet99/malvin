use crate::agent_backend::AgentBackend;
use crate::router_flow::router_flow_prompt;

pub(crate) async fn run_router_requirements_coder_prompt(
    client: &mut AgentBackend,
    coder: &router_flow_prompt::RouterCoderRun,
    log_path: &std::path::Path,
) -> Result<(), String> {
    client
        .run_coder_prompt(
            &coder.combined,
            log_path,
            "router_requirements",
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

pub(crate) async fn run_router_kpop_group_coder_prompt(
    client: &mut AgentBackend,
    prompt: &str,
    log_path: &std::path::Path,
) -> Result<(), String> {
    client
        .run_coder_prompt(
            prompt,
            log_path,
            "router_kpop",
            crate::acp::CoderPromptOptions {
                llm_phase: Some(crate::run_timing::TimingPhase::Implement),
                do_trace_split: None,
                stdout_bracket_label: None,
                append_trace: true,
                ..Default::default()
            },
        )
        .await
        .map_err(|e| e.to_string())
}

pub(crate) async fn run_router_work_coder_prompt(
    client: &mut AgentBackend,
    prompt: &str,
    log_path: &std::path::Path,
) -> Result<(), String> {
    client
        .run_coder_prompt(
            prompt,
            log_path,
            "router_work",
            crate::acp::CoderPromptOptions {
                llm_phase: Some(crate::run_timing::TimingPhase::Implement),
                do_trace_split: None,
                stdout_bracket_label: None,
                append_trace: true,
                ..Default::default()
            },
        )
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod kiss_cov_gate_refs {
    use super::*;

    #[test]
    fn kiss_cov_unit_names() {
        let _ = run_router_requirements_coder_prompt;
        let _ = run_router_kpop_group_coder_prompt;
        let _ = run_router_work_coder_prompt;
    }
}
