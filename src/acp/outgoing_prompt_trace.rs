// Outgoing ACP trace layout for `session/prompt` (uniform stem vs `malvin do` split).

/// Single-stem trace (`>who`) plus optional override for the stdout `[label...]` line.
pub struct UniformOutgoingTrace<'a> {
    pub trace_who: &'a str,
    pub stdout_bracket_label: Option<&'a str>,
}

/// Configuration options for [`crate::AgentClient::run_coder_prompt`].
#[derive(Default)]
pub struct CoderPromptOptions<'a> {
    /// LLM phase for run timing (None to skip timing).
    pub llm_phase: Option<crate::run_timing::TimingPhase>,
    /// When Some, `malvin do` uses [`DoPromptTraceSplit`] to build the payload; the on-disk trace
    /// file records the composed prompt as plain lines (see `trace_write_outgoing_prompt_do` in
    /// `session_trace.rs`), not per-segment `>header` / `>prompt` lines.
    pub do_trace_split: Option<(&'a str, &'a str)>,
    /// Override for the stdout `[label...]` bracket line (defaults to `who` if None).
    pub stdout_bracket_label: Option<&'a str>,
}

/// How outgoing `session/prompt` text is mirrored to the trace file and (when tee is on) stdout.
pub enum OutgoingPromptTrace<'a> {
    Uniform(UniformOutgoingTrace<'a>),
    /// `malvin do`: [`DoPromptTraceSplit`] supplies header and user segments that are
    /// composed into one payload; the trace file holds that payload as plain lines (tests in
    /// `session_trace.rs` lock this). Stdout and `prompts.log` use the directional `>do` stem.
    DoSplit(DoPromptTraceSplit<'a>),
}

/// Segments used to build the full `malvin do` `session/prompt` payload (header, user).
pub struct DoPromptTraceSplit<'a> {
    pub header: &'a str,
    pub user: &'a str,
}

#[test]
fn coder_prompt_options_default_and_trace_variants_construct() {
    let _ = CoderPromptOptions::default();
    let uniform = OutgoingPromptTrace::Uniform(UniformOutgoingTrace {
        trace_who: "coder",
        stdout_bracket_label: None,
    });
    let split = OutgoingPromptTrace::DoSplit(DoPromptTraceSplit {
        header: "h",
        user: "u",
    });
    assert!(matches!(uniform, OutgoingPromptTrace::Uniform(_)));
    assert!(matches!(split, OutgoingPromptTrace::DoSplit(_)));
}
