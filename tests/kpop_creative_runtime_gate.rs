#[test]
fn kpop_p_creative_runtime_gate_contract() {
    assert!(!malvin::kpop_creative_enabled(0.0));
    assert!(!malvin::kpop_creative_enabled(-0.1));
    assert!(!malvin::kpop_creative_enabled(f64::INFINITY));
    assert!(!malvin::kpop_creative_enabled(f64::NAN));
    assert!(malvin::kpop_creative_enabled(0.1));
}
