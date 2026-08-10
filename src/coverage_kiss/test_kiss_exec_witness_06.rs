//! Generated executable-call witnesses for kiss static coverage (bridge_sdk merge).
//! Orphan test file (not in the crate module tree); kiss-analyzed only.

#[test]
fn kiss_cov_bridge_sdk_spawn_names_cursor() {
    cursor_spawn_bridge();
    cursor_open_bridge_session();
    CursorChildStdio();
    cursor_take_stdio();
    cursor_note_sandbox();
    cursor_assemble_session();
    cursor_resolve_node_and_bridge();
    cursor_build_bridge_command();
}

#[test]
fn kiss_cov_bridge_sdk_spawn_names_prime() {
    prime_spawn_bridge();
    prime_open_bridge_session();
    PrimeChildStdio();
    prime_take_stdio();
    prime_note_sandbox();
    prime_assemble_session();
    prime_resolve_node_and_bridge();
    prime_build_bridge_command();
    scrub_cursor_keys();
    apply_node_compile_cache();
}

#[test]
fn kiss_cov_bridge_sdk_shared_type_names() {
    BridgeKind();
    SdkClientInit();
    CreateArgs();
    ResumeArgs();
    SdkClient();
    from_init();
    sync_timing_to_open_session();
}
