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
    scrub_cursor_keys();
    apply_node_compile_cache();
}

#[test]
fn kiss_cov_bridge_sdk_shared_type_names() {
    BridgeKind();
    BridgeWire();
    NodeBridge();
    PiRpc();
    SdkClientInit();
    CreateArgs();
    ResumeArgs();
    SdkClient();
    from_init();
    sync_timing_to_open_session();
    cursor_sdk_marker_present();
    encode_request();
    drain_until_run_done();
    finish_run_done();
    cursor_mock_write_line();
    kiss_cov_bridge_path_name_batch();
}

fn kiss_cov_bridge_path_name_batch() {
    cursor_first_ready_bridge_js();
    cursor_first_any_bridge_js();
    cursor_first_ready_models_js();
    cursor_first_any_models_js();
    cursor_candidate_roots();
}

#[test]
fn kiss_cov_sdk_bridge_build_install_names() {
    fnv1a64();
    Bridge();
    BRIDGES();
    run_build_script();
    emit_rerun_if_changed();
    ensure_bridge();
    in_tree_bridge_ready();
    share_bridge_ready();
    install_npm_deps();
    verify_install();
    write_stamp();
    sdk_share_dir();
    copy_dir_recursive();
    sync_bridge_payload();
    copy_build_sources();
    resolve_npm();
    which();
    check_node_version();
    parse_node_version();
    run_npm();
}
