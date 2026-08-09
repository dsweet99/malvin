//! Options for [`crate::agent_backend::AgentBackend::run_coder_prompt`].

/// Configuration options for coder prompt turns.
#[derive(Default)]
pub struct CoderPromptOptions<'a> {
    /// LLM phase for run timing (None to skip timing).
    pub llm_phase: Option<crate::run_timing::TimingPhase>,
    /// When Some, `malvin --do` uses header/user split for the composed prompt payload.
    pub do_trace_split: Option<(&'a str, &'a str)>,
    /// Override for the stdout `[label...]` bracket line (defaults to `who` if None).
    pub stdout_bracket_label: Option<&'a str>,
    /// When true, skip client-level prompt retries (gate kpop outer loop owns retries).
    pub single_attempt: bool,
    /// When true, append to the trace file instead of truncating (multi-turn sessions).
    pub append_trace: bool,
}

#[test]
fn coder_prompt_options_default_constructs() {
    let opts = CoderPromptOptions::default();
    assert!(opts.llm_phase.is_none());
    assert!(opts.do_trace_split.is_none());
    assert!(opts.stdout_bracket_label.is_none());
    assert!(!opts.single_attempt);
    assert!(!opts.append_trace);
}
