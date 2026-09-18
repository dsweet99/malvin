#[test]
fn kiss_cov_remaining_shared_helpers() {
    use crate::coder_prompt_phase::MiniPhase;
    assert_eq!(MiniPhase::Investigate.as_str(), "investigate");
    let _ = (MiniPhase::WindDown, MiniPhase::Terminal);
    let parsed = crate::model_id::parse_model_id("cursor:auto").expect("parse");
    let _ = parsed.backend;
    let _ = parsed.canonical();
}
