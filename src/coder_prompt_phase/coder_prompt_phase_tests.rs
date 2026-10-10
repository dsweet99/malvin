use super::MiniPhase;
#[test]
fn mini_phase_as_str_contract_and_mini_phase_variants_exist() {
    {
        assert_eq!(MiniPhase::Investigate.as_str(), "investigate");
        assert_eq!(MiniPhase::WindDown.as_str(), "wind_down");
        assert_eq!(MiniPhase::Terminal.as_str(), "terminal");
    }
    {
        let _ = (
            MiniPhase::Investigate,
            MiniPhase::WindDown,
            MiniPhase::Terminal,
        );
    }
}
