#[path = "https_trust/harness.rs"]
mod https_trust;

#[test]
fn system_trust_accepts_private_ca() {
    https_trust::assert_system_trust_accepts_private_ca();
}
