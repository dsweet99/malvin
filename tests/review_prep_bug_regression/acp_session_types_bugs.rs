use super::helpers::manifest_root;

#[test]
fn session_types_rs_must_not_host_test_only_symbol_names() {
    let session_types = std::fs::read_to_string(manifest_root().join("src/acp/session_types.rs"))
        .expect("read session_types.rs");
    assert!(
        !session_types.contains("response_tx_oneshot_channel_constructible"),
        "bug: production session_types.rs must not host a test-only symbol name"
    );
}
