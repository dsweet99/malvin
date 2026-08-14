
#[derive(Default)]
pub struct CoderPromptOptions<'a> {
    pub llm_phase: Option<crate::run_timing::TimingPhase>,
    pub do_trace_split: Option<(&'a str, &'a str)>,
    pub stdout_bracket_label: Option<&'a str>,
    pub single_attempt: bool,
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
