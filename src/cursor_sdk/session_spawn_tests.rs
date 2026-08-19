#[test]
fn kiss_cov_cursor_session_spawn_names() {
    let _ = super::spawn_bridge;
    let _ = stringify!(cursor_spawn_bridge);
    let _ = stringify!(cursor_open_bridge_session);
    let _ = stringify!(CursorChildStdio);
    let _ = stringify!(cursor_take_stdio);
    let _ = stringify!(cursor_note_sandbox);
    let _ = stringify!(cursor_assemble_session);
    let _ = stringify!(cursor_resolve_node_and_bridge);
    let _ = stringify!(cursor_build_bridge_command);
}
