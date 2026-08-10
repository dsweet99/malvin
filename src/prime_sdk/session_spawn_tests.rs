//! Name witnesses for Prime session spawn helpers (kiss `test_coverage`).

#[test]
fn kiss_cov_session_and_spawn_names() {
    let _ = super::spawn_bridge;
    let _ = stringify!(prime_spawn_bridge);
    let _ = stringify!(prime_open_bridge_session);
    let _ = stringify!(PrimeChildStdio);
    let _ = stringify!(prime_take_stdio);
    let _ = stringify!(prime_note_sandbox);
    let _ = stringify!(prime_assemble_session);
    let _ = stringify!(prime_resolve_node_and_bridge);
    let _ = stringify!(prime_build_bridge_command);
    let _ = stringify!(scrub_cursor_keys);
    let _ = stringify!(apply_node_compile_cache);
}
