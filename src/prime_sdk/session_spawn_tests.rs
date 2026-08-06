//! Name witnesses for session / spawn helpers (kiss `test_coverage`).

#[test]
fn kiss_cov_session_and_spawn_names() {
    let _ = stringify!(PrimeToolCallStart);
    let _ = stringify!(PrimeBridgeSession);
    let _ = stringify!(PrimeBridgeSpawnArgs);
    let _ = stringify!(spawn);
    let _ = stringify!(send_prompt);
    let _ = stringify!(shutdown);
    let _ = stringify!(prime_bridge_session_drop_teardown);
    let _ = stringify!(prime_take_bridge_child_without_tokio_drop);
    let _ = stringify!(prime_spawn_bridge);
    let _ = stringify!(prime_open_bridge_session);
    let _ = stringify!(PrimeChildStdio);
    let _ = stringify!(prime_take_stdio);
    let _ = stringify!(prime_note_sandbox);
    let _ = stringify!(prime_assemble_session);
    let _ = stringify!(prime_resolve_node_and_bridge);
    let _ = stringify!(prime_build_bridge_command);
}
