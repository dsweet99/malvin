//! Outgoing ACP trace layout for `session/prompt` (uniform stem vs `malvin do` split).

/// Single-stem trace (`>who`) plus optional override for the stdout `[label...]` line.
pub struct UniformOutgoingTrace<'a> {
    pub trace_who: &'a str,
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
}
