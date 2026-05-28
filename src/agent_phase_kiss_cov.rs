#[test]
fn kiss_cov_agent_phase_units() {
    let _ = crate::agent_phase::print_done_with_reporting_phase;
    crate::agent_phase::with_state(|_| {});
    let _: Option<crate::agent_phase::ToolKind> = None;
}
