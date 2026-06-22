use std::path::Path;

use crate::acp::{
    outgoing_prompt_trace::DoPromptTraceSplit, retries_noun, AcpSession, AgentError,
};

pub(crate) struct CoderSessionPromptDispatch<'a> {
    pub session: &'a AcpSession,
    pub full_prompt: &'a str,
    pub log_path: &'a Path,
    pub who: &'a str,
    pub do_trace_split: Option<(&'a str, &'a str)>,
    pub stdout_bracket_label: Option<&'a str>,
}

pub(crate) async fn dispatch_coder_session_prompt(
    dispatch: &CoderSessionPromptDispatch<'_>,
) -> Result<(), String> {
    match dispatch.do_trace_split {
        None => {
            dispatch
                .session
                .prompt(
                    dispatch.full_prompt,
                    dispatch.log_path,
                    dispatch.who,
                    dispatch.stdout_bracket_label,
                )
                .await
        }
        Some((header, user)) => {
            dispatch
                .session
                .prompt_do_trace_split(
                    dispatch.full_prompt,
                    dispatch.log_path,
                    DoPromptTraceSplit { header, user },
                )
                .await
        }
    }
}

pub(crate) fn coder_prompt_exhausted_error(attempts_used: u32, last_error: String) -> AgentError {
    let retries = attempts_used.saturating_sub(1);
    let noun = retries_noun(retries);
    AgentError(format!(
        "agent acp (coder prompt) failed after {retries} {noun}. Last error:\n{last_error}"
    ))
}

pub(crate) fn record_coder_prompt_llm_timing(
    timing: Option<&std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
    llm_phase: Option<crate::run_timing::TimingPhase>,
    elapsed: std::time::Duration,
) {
    if let Some(ph) = llm_phase {
        crate::run_timing::record_llm(timing, ph, elapsed);
    }
}

#[test]
fn coder_prompt_exhausted_error_reports_last_error() {
    let err = coder_prompt_exhausted_error(2, "transport reset".into());
    let msg = err.to_string();
    assert!(msg.contains("transport reset"));
    assert!(msg.contains("agent acp (coder prompt) failed"));
}

#[test]
fn record_coder_prompt_llm_timing_no_phase_is_noop() {
    record_coder_prompt_llm_timing(None, None, std::time::Duration::from_millis(1));
}

#[test]
fn coder_session_prompt_dispatch_type_witness() {
    let _ = stringify!(CoderSessionPromptDispatch);
    let _ = stringify!(dispatch_coder_session_prompt);
    let _: Option<CoderSessionPromptDispatch> = None;
    let _ = dispatch_coder_session_prompt;
}
#[cfg(test)]
#[path = "client_impl_prompt_dispatch_test.rs"]
mod client_impl_prompt_dispatch_test;
