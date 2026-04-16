//! Outgoing ACP trace layout for `session/prompt` (uniform stem vs `malvin do` split).

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
    /// When true, skip prepending repo style even on first turn.
    pub skip_repo_style: bool,
    /// When Some, split trace into `>style` / `>header` / `>prompt` segments for `malvin do`.
    pub do_trace_split: Option<(&'a str, &'a str)>,
    /// Override for the stdout `[label...]` bracket line (defaults to `who` if None).
    pub stdout_bracket_label: Option<&'a str>,
}

/// How outgoing `session/prompt` text is mirrored to the trace file and (when tee is on) stdout.
pub enum OutgoingPromptTrace<'a> {
    Uniform(UniformOutgoingTrace<'a>),
    /// `malvin do`: split optional injected repo style, `header.md`, and user request in the trace.
    DoSplit(DoPromptTraceSplit<'a>),
}

/// Segments for `malvin do` outgoing trace (`>style` / `>header` / `>prompt`).
pub struct DoPromptTraceSplit<'a> {
    pub style_text: Option<&'a str>,
    pub header: &'a str,
    pub user: &'a str,
}

#[test]
fn kiss_stringify_outgoing_prompt_trace() {
    let _ = stringify!(UniformOutgoingTrace);
    let _ = stringify!(OutgoingPromptTrace::Uniform);
    let _ = stringify!(OutgoingPromptTrace::DoSplit);
    let _ = stringify!(DoPromptTraceSplit);
    let _ = stringify!(CoderPromptOptions);
}
