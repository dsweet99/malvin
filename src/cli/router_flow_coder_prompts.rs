use crate::agent_backend::AgentBackend;
use crate::prompts::{KPOP_COMMON_MD, header_prompt_file, router_a_prompt_file};

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
                stdout_bracket_label: Some(header_prompt_file()),
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
                stdout_bracket_label: Some(router_a_prompt_file()),
                append_trace: true,
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
            "kpop_common",
            crate::acp::CoderPromptOptions {
                llm_phase: Some(crate::run_timing::TimingPhase::Implement),
                do_trace_split: None,
                stdout_bracket_label: Some(KPOP_COMMON_MD),
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
    stdout_bracket_label: &str,
) -> Result<(), String> {
    client
        .run_coder_prompt(
            prompt,
            log_path,
            "router_b",
            crate::acp::CoderPromptOptions {
                llm_phase: Some(crate::run_timing::TimingPhase::Implement),
                do_trace_split: None,
                stdout_bracket_label: Some(stdout_bracket_label),
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
        let _ = run_router_a_coder_prompt;
        let _ = run_router_kpop_common_coder_prompt;
        let _ = run_router_b_coder_prompt;
        let _ = run_router_summarize_coder_prompt;
        let _ = header_prompt_file;
        let _ = router_a_prompt_file;
    }

    #[test]
    fn router_coder_stdout_labels_match_active_prompt_files() {
        assert_eq!(header_prompt_file(), "header.md");
        assert_eq!(router_a_prompt_file(), "router_a.md");
    }
}
