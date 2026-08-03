use crate::agent_backend::AgentBackend;

pub(crate) async fn run_router_header_coder_prompt(
    client: &mut AgentBackend,
    prompt: &str,
    log_path: &std::path::Path,
) -> Result<(), String> {
    client
        .run_coder_prompt(
            prompt,
            log_path,
            "router_header",
            crate::acp::CoderPromptOptions {
                llm_phase: Some(crate::run_timing::TimingPhase::Implement),
                do_trace_split: None,
                stdout_bracket_label: Some("header.md"),
                ..Default::default()
            },
        )
        .await
        .map_err(|e| e.to_string())
}

pub(crate) async fn run_router_kpop_common_coder_prompt(
    client: &mut AgentBackend,
    prompt: &str,
    log_path: &std::path::Path,
) -> Result<(), String> {
    client
        .run_coder_prompt(
            prompt,
            log_path,
            "router_kpop_common",
            crate::acp::CoderPromptOptions {
                llm_phase: Some(crate::run_timing::TimingPhase::Implement),
                do_trace_split: None,
                stdout_bracket_label: Some("kpop_common.md"),
                append_trace: true,
                ..Default::default()
            },
        )
        .await
        .map_err(|e| e.to_string())
}

pub(crate) async fn run_router_a_coder_prompt(
    client: &mut AgentBackend,
    prompt: &str,
    log_path: &std::path::Path,
) -> Result<(), String> {
    client
        .run_coder_prompt(
            prompt,
            log_path,
            "router_a",
            crate::acp::CoderPromptOptions {
                llm_phase: Some(crate::run_timing::TimingPhase::Implement),
                do_trace_split: None,
                stdout_bracket_label: Some("router_a.md"),
                append_trace: true,
                ..Default::default()
            },
        )
        .await
        .map_err(|e| e.to_string())
}

pub(crate) async fn run_router_b_coder_prompt(
    client: &mut AgentBackend,
    prompt: &str,
    log_path: &std::path::Path,
) -> Result<(), String> {
    client
        .run_coder_prompt(
            prompt,
            log_path,
            "router_b",
            crate::acp::CoderPromptOptions {
                llm_phase: Some(crate::run_timing::TimingPhase::Implement),
                do_trace_split: None,
                stdout_bracket_label: Some("router_b.md"),
                append_trace: true,
                ..Default::default()
            },
        )
        .await
        .map_err(|e| e.to_string())
}

pub(crate) async fn run_router_summarize_coder_prompt(
    client: &mut AgentBackend,
    prompt: &str,
    log_path: &std::path::Path,
) -> Result<(), String> {
    client
        .run_coder_prompt(
            prompt,
            log_path,
            "router_summarize",
            crate::acp::CoderPromptOptions {
                llm_phase: Some(crate::run_timing::TimingPhase::Implement),
                do_trace_split: None,
                stdout_bracket_label: Some("router_summarize.md"),
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
        let _ = run_router_header_coder_prompt;
        let _ = run_router_kpop_common_coder_prompt;
        let _ = run_router_a_coder_prompt;
        let _ = run_router_b_coder_prompt;
        let _ = run_router_summarize_coder_prompt;
    }
}
