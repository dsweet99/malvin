use super::*;

#[test]
fn prefer_gate_outcome_surfaces_restore_when_gate_passed() {
    let err = prefer_gate_outcome_over_post_gate_cleanup(
        Ok(()),
        Err("malvin_checks restore: boom".into()),
    )
    .unwrap_err();
    assert!(err.contains("malvin_checks restore"));
}
