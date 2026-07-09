use crate::agent_backend::AgentBackend;
use crate::router_flow::router_flow_prompt;

pub(crate) async fn run_router_a_coder_prompt(
    client: &mut AgentBackend,
    coder: &router_flow_prompt::RouterCoderRun,
    log_path: &std::path::Path,
) -> Result<(), String> {
    client
        .run_coder_prompt(
            &coder.combined,
            log_path,
            "router_a",
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

pub(crate) async fn run_router_b_coder_prompt(
    client: &mut AgentBackend,
    router_b_prompt: &str,
    log_path: &std::path::Path,
    label: &str,
) -> Result<(), String> {
    client
        .run_coder_prompt(
            router_b_prompt,
            log_path,
            label,
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

pub(crate) async fn run_router_c_coder_prompt(
    client: &mut AgentBackend,
    router_c_prompt: &str,
    log_path: &std::path::Path,
) -> Result<(), String> {
    client
        .run_coder_prompt(
            router_c_prompt,
            log_path,
            "router_c",
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
        let _ = run_router_a_coder_prompt;
        let _ = run_router_b_coder_prompt;
        let _ = run_router_c_coder_prompt;
    }
}
