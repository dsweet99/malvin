use super::MiniPhase;

#[test]
fn mini_phase_as_str_contract() {
    assert_eq!(MiniPhase::Investigate.as_str(), "investigate");
    assert_eq!(MiniPhase::WindDown.as_str(), "wind_down");
    assert_eq!(MiniPhase::Terminal.as_str(), "terminal");
}

#[test]
fn mini_phase_variants_exist() {
    let _ = (
        MiniPhase::Investigate,
        MiniPhase::WindDown,
        MiniPhase::Terminal,
    );
}
