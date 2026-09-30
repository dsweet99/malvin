#[path = "https_trust/harness.rs"]
mod https_trust;

#[test]
fn webpki_rejects_private_ca() {
    https_trust::assert_webpki_rejects_private_ca();
}
