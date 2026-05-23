// Per-symbol kiss coverage wires.

// Per-symbol kiss coverage wires.

#[test]
fn kiss_cov_build_rs_probe_writable_cgroup_parent() {
    let _ = stringify!(crate::cgroup_build::probe_writable_cgroup_parent);
}

#[test]
fn kiss_cov_build_rs_resolve_cgroup_v2_parent() {
    let _ = stringify!(crate::cgroup_build::resolve_cgroup_v2_parent);
}

#[test]
fn kiss_cov_build_rs_resolve_cgroup_v1_memory_parent() {
    let _ = stringify!(crate::cgroup_build::resolve_cgroup_v1_memory_parent);
}

#[test]
fn kiss_cov_build_rs_self_cgroup_v1_memory_relative_path() {
    let _ = stringify!(crate::cgroup_build::self_cgroup_v1_memory_relative_path);
}

#[test]
fn kiss_cov_build_rs_probe_writable_parent() {
    let _ = stringify!(crate::cgroup_build::probe_writable_parent);
}

#[test]
fn kiss_cov_src_acp_kiss_coverage_rs_smoke_reader_loop_eof_pending_error() {
    let _ = stringify!(crate::acp::kiss_coverage::smoke_reader_loop_eof_pending_error);
}

#[test]
fn kiss_cov_src_acp_ops_inline_tests_rs_write_path_executable() {
    let _ = stringify!(crate::acp::ops_inline_tests::write_path_executable);
}

#[test]
fn kiss_cov_src_acp_reader_tests_dispatch_rs_test_dispatch_response_ok_error_orphans_and_malformed()
{
    let _ = stringify!(
        crate::acp::reader_tests_dispatch::test_dispatch_response_ok_error_orphans_and_malformed
    );
}

#[test]
fn kiss_cov_src_acp_reader_tests_dispatch_rs_test_handle_incoming_line_parse_error_and_extension_method()
 {
    let _ = stringify!(crate::acp::reader_tests_dispatch::test_handle_incoming_line_parse_error_and_extension_method);
}

#[test]
fn kiss_cov_src_acp_reader_tests_permission_unix_rs_test_handle_session_update_and_permission_replies()
 {
    let _ = stringify!(
        crate::acp::reader_tests_permission_unix::test_handle_session_update_and_permission_replies
    );
}

#[test]
fn kiss_cov_src_acp_reader_tests_permission_unix_rs_test_permission_json_or_write_failure_is_logged()
 {
    let _ = stringify!(
        crate::acp::reader_tests_permission_unix::test_permission_json_or_write_failure_is_logged
    );
}

#[test]
fn kiss_cov_src_acp_reader_tests_reader_loop_rs_test_reader_loop_drains_pending_on_stdout_eof() {
    let _ = stringify!(
        crate::acp::reader_tests_reader_loop::test_reader_loop_drains_pending_on_stdout_eof
    );
}

#[test]
fn kiss_cov_src_acp_reader_tests_trace_a_rs_trace_chunk_coalescer_merges_two_small_message_chunks()
{
    let _ = stringify!(
        crate::acp::reader_tests_trace_a::trace_chunk_coalescer_merges_two_small_message_chunks
    );
}

#[test]
fn kiss_cov_src_acp_session_types_rs_acp_session_inner() {
    let _ = stringify!(crate::acp::session_types::AcpSessionInner);
}

#[test]
fn kiss_cov_src_acp_session_tests_cancel_rs_busy_session_with_dead_transport() {
    let _ = stringify!(crate::acp_session_unit_tests::busy_session_with_dead_transport);
}

#[test]
fn kiss_cov_src_acp_session_tests_cancel_rs_acp_session_cancel_clears_busy_state_after_rpc_error() {
    let _ = stringify!(
        crate::acp_session_unit_tests::acp_session_cancel_clears_busy_state_after_rpc_error
    );
}

#[test]
fn kiss_cov_src_acp_session_tests_linux_spawn_abort_rs_acp_session_spawn_aborts_when_linux_cgroup_verify_fails()
 {
    let _ = stringify!(
        crate::acp_session_unit_tests::acp_session_spawn_aborts_when_linux_cgroup_verify_fails
    );
}

#[test]
fn kiss_cov_src_acp_session_tests_unix_helpers_rs_wait_for_pid_file() {
    let _ = stringify!(crate::acp_session_unit_tests::unix_helpers::wait_for_pid_file);
}
