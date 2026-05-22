use super::multiturn::{NextStep, Phase};

#[test]
fn smoke_multiturn_phase_and_next_step_variants() {
    let stop = NextStep::Stop;
    assert!(matches!(stop, NextStep::Stop));
    let phase = Phase::KpopBlock {
        target_n: 1,
        hypotheses_before: 0,
        attempts: 0,
    };
    assert!(matches!(phase, Phase::KpopBlock { .. }));
    let mbc2 = Phase::Mbc2 {
        baseline: 0,
        sent: 0,
    };
    assert!(matches!(mbc2, Phase::Mbc2 { .. }));
    let emit = NextStep::Emit(crate::multiturn_prompt::MultiturnPrompt::KpopBlock(
        "x".into(),
    ));
    assert!(matches!(emit, NextStep::Emit(_)));
}
