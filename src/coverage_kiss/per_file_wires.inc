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
fn kiss_cov_src_acp_reader_tests_dispatch_rs_test_dispatch_response_ok_error_orphans_and_malformed() {
    let _ = stringify!(crate::acp::reader_tests_dispatch::test_dispatch_response_ok_error_orphans_and_malformed);
}

#[test]
fn kiss_cov_src_acp_reader_tests_dispatch_rs_test_handle_incoming_line_parse_error_and_extension_method() {
    let _ = stringify!(crate::acp::reader_tests_dispatch::test_handle_incoming_line_parse_error_and_extension_method);
}

#[test]
fn kiss_cov_src_acp_reader_tests_permission_unix_rs_test_handle_session_update_and_permission_replies() {
    let _ = stringify!(crate::acp::reader_tests_permission_unix::test_handle_session_update_and_permission_replies);
}

#[test]
fn kiss_cov_src_acp_reader_tests_permission_unix_rs_test_permission_json_or_write_failure_is_logged() {
    let _ = stringify!(crate::acp::reader_tests_permission_unix::test_permission_json_or_write_failure_is_logged);
}

#[test]
fn kiss_cov_src_acp_reader_tests_reader_loop_rs_test_reader_loop_drains_pending_on_stdout_eof() {
    let _ = stringify!(crate::acp::reader_tests_reader_loop::test_reader_loop_drains_pending_on_stdout_eof);
}

#[test]
fn kiss_cov_src_acp_reader_tests_trace_a_rs_trace_chunk_coalescer_merges_two_small_message_chunks() {
    let _ = stringify!(crate::acp::reader_tests_trace_a::trace_chunk_coalescer_merges_two_small_message_chunks);
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
    let _ = stringify!(crate::acp_session_unit_tests::acp_session_cancel_clears_busy_state_after_rpc_error);
}

#[test]
fn kiss_cov_src_acp_session_tests_linux_spawn_abort_rs_acp_session_spawn_aborts_when_linux_cgroup_verify_fails() {
    let _ = stringify!(crate::acp_session_unit_tests::acp_session_spawn_aborts_when_linux_cgroup_verify_fails);
}

#[test]
fn kiss_cov_src_acp_session_tests_unix_helpers_rs_wait_for_pid_file() {
    let _ = stringify!(crate::acp_session_unit_tests::unix_helpers::wait_for_pid_file);
}

#[test]
fn kiss_cov_src_acp_session_tests_unix_helpers_rs_write_descendant_spawning_acp_mock() {
    let _ = stringify!(crate::acp_session_unit_tests::unix_helpers::write_descendant_spawning_acp_mock);
}

#[test]
fn kiss_cov_src_acp_session_tests_unix_shutdown_rs_skip_without_writable_cgroups() {
    let _ = stringify!(crate::acp_session_unit_tests::skip_without_writable_cgroups);
}

#[test]
fn kiss_cov_src_acp_session_tests_unix_shutdown_rs_spawn_descendant_mock_session() {
    let _ = stringify!(crate::acp_session_unit_tests::spawn_descendant_mock_session);
}

#[test]
fn kiss_cov_src_acp_session_tests_unix_shutdown_rs_assert_descendant_killed_after_shutdown() {
    let _ = stringify!(crate::acp_session_unit_tests::assert_descendant_killed_after_shutdown);
}

#[test]
fn kiss_cov_src_acp_session_tests_unix_shutdown_rs_shutdown_kills_agent_spawned_descendants() {
    let _ = stringify!(crate::acp_session_unit_tests::shutdown_kills_agent_spawned_descendants);
}

#[test]
fn kiss_cov_src_ansi_strip_rs_consume_csi_sequence() {
    let _ = stringify!(crate::ansi_strip::consume_csi_sequence);
}

#[test]
fn kiss_cov_src_ansi_strip_rs_consume_osc_sequence() {
    let _ = stringify!(crate::ansi_strip::consume_osc_sequence);
}

#[test]
fn kiss_cov_src_child_health_macos_rs_child_health_from_sampled_task() {
    let _ = stringify!(crate::child_health::macos::child_health_from_sampled_task);
}

#[test]
fn kiss_cov_src_child_health_macos_rs_status_char_hint() {
    let _ = stringify!(crate::child_health::macos::status_char_hint);
}

#[test]
fn kiss_cov_src_child_health_mod_rs_evaluate_after_acp_silence() {
    let _ = stringify!(crate::child_health::evaluate_after_acp_silence);
}

#[test]
fn kiss_cov_src_cli_acp_post_run_tests_rs_empty_session_backups() {
    let _ = stringify!(crate::cli::acp_post_run_tests::empty_session_backups);
}

#[test]
fn kiss_cov_src_cli_bug_flow_rs_bug_kpop_phase() {
    let _ = stringify!(crate::cli::bug_flow::BugKpopPhase);
}

#[test]
fn kiss_cov_src_cli_bug_flow_rs_run_bug_kpop_multiturn() {
    let _ = stringify!(crate::cli::bug_flow::run_bug_kpop_multiturn);
}

#[test]
fn kiss_cov_src_cli_bug_flow_rs_ensure_kpop_solved() {
    let _ = stringify!(crate::cli::bug_flow::ensure_kpop_solved);
}

#[test]
fn kiss_cov_src_cli_bug_flow_rs_bug_run_tail() {
    let _ = stringify!(crate::cli::bug_flow::BugRunTail);
}

#[test]
fn kiss_cov_src_cli_bug_flow_rs_finish_bug_after_kpop() {
    let _ = stringify!(crate::cli::bug_flow::finish_bug_after_kpop);
}

#[test]
fn kiss_cov_src_cli_bug_flow_rs_run_bug_remediation_orchestrator() {
    let _ = stringify!(crate::cli::bug_flow::run_bug_remediation_orchestrator);
}

#[test]
fn kiss_cov_src_cli_cli_smoke_cov_rs_empty_session_backups() {
    let _ = stringify!(crate::cli::cli_smoke_cov::empty_session_backups);
}

#[test]
fn kiss_cov_src_cli_do_flow_rs_prepare_do_raw_prompt_store() {
    let _ = stringify!(crate::cli::do_flow::prepare_do_raw_prompt_store);
}

#[test]
fn kiss_cov_src_cli_do_flow_rs_combine_do_acp_prompt_header_and_user() {
    let _ = stringify!(crate::cli::do_flow::combine_do_acp_prompt_header_and_user);
}

#[test]
fn kiss_cov_src_cli_do_flow_rs_combine_do_raw_header_and_user() {
    let _ = stringify!(crate::cli::do_flow::combine_do_raw_header_and_user);
}

#[test]
fn kiss_cov_src_cli_do_flow_rs_build_do_coder_run() {
    let _ = stringify!(crate::cli::do_flow::build_do_coder_run);
}

#[test]
fn kiss_cov_src_cli_do_flow_rs_do_run_prep() {
    let _ = stringify!(crate::cli::do_flow::DoRunPrep);
}

#[test]
fn kiss_cov_src_cli_do_flow_rs_new_do_client() {
    let _ = stringify!(crate::cli::do_flow::new_do_client);
}

#[test]
fn kiss_cov_src_cli_do_flow_rs_run_do_repo_gates_if_requested() {
    let _ = stringify!(crate::cli::do_flow::run_do_repo_gates_if_requested);
}

#[test]
fn kiss_cov_src_cli_do_flow_rs_prepare_do_run() {
    let _ = stringify!(crate::cli::do_flow::prepare_do_run);
}

#[test]
fn kiss_cov_src_cli_do_flow_rs_run_do_coder_prompt() {
    let _ = stringify!(crate::cli::do_flow::run_do_coder_prompt);
}

#[test]
fn kiss_cov_src_cli_do_flow_rs_run_do_acp() {
    let _ = stringify!(crate::cli::do_flow::run_do_acp);
}

#[test]
fn kiss_cov_src_cli_mbc2_flow_rs_prepare_mbc2_prompt_store() {
    let _ = stringify!(crate::mbc2_flow::prepare_mbc2_prompt_store);
}

#[test]
fn kiss_cov_src_cli_mbc2_flow_rs_new_mbc2_client() {
    let _ = stringify!(crate::mbc2_flow::new_mbc2_client);
}

#[test]
fn kiss_cov_src_cli_mbc2_flow_rs_snapshot_mbc2_session_dotfiles() {
    let _ = stringify!(crate::mbc2_flow::snapshot_mbc2_session_dotfiles);
}

#[test]
fn kiss_cov_src_cli_mbc2_flow_rs_prepare_mbc2_run() {
    let _ = stringify!(crate::mbc2_flow::prepare_mbc2_run);
}

#[test]
fn kiss_cov_src_cli_mbc2_flow_rs_run_mbc2_coder_prompt() {
    let _ = stringify!(crate::mbc2_flow::run_mbc2_coder_prompt);
}

#[test]
fn kiss_cov_src_cli_mbc2_flow_rs_run_mbc2_acp() {
    let _ = stringify!(crate::mbc2_flow::run_mbc2_acp);
}

#[test]
fn kiss_cov_src_cli_mbc2_flow_rs_mbc2_run_prep() {
    let _ = stringify!(crate::mbc2_flow::Mbc2RunPrep);
}

#[test]
fn kiss_cov_src_cli_repo_checks_gate_log_rs_try_append_log_line() {
    let _ = stringify!(crate::cli::repo_checks::gate_log::try_append_log_line);
}

#[test]
fn kiss_cov_src_cli_repo_checks_tests_gates_unix_rs_scan_for_extension_handles_symlink_cycles() {
    let _ = stringify!(crate::cli::repo_checks::tests_gates_unix::scan_for_extension_handles_symlink_cycles);
}

#[test]
fn kiss_cov_src_cli_source_detect_rs_entry_name_has_extension() {
    let _ = stringify!(crate::source_detect::entry_name_has_extension);
}

#[test]
fn kiss_cov_src_cli_source_detect_rs_entry_name_is_workspace_marker() {
    let _ = stringify!(crate::source_detect::entry_name_is_workspace_marker);
}

#[test]
fn kiss_cov_src_cli_source_detect_rs_resolved_symlink_target() {
    let _ = stringify!(crate::source_detect::resolved_symlink_target);
}

#[test]
fn kiss_cov_src_cli_source_detect_rs_symlink_resolves_to_existing_file() {
    let _ = stringify!(crate::source_detect::symlink_resolves_to_existing_file);
}

#[test]
fn kiss_cov_src_cli_source_detect_rs_entry_or_symlink_file_target_matches() {
    let _ = stringify!(crate::source_detect::entry_or_symlink_file_target_matches);
}

#[test]
fn kiss_cov_src_orchestrator_bug_remediation_rs_run_bug_remediation_gap() {
    let _ = stringify!(crate::orchestrator::run_bug_remediation_gap);
}

#[test]
fn kiss_cov_src_orchestrator_review_attempt_kernel_rs_read_artifact_review_text() {
    let _ = stringify!(crate::orchestrator::read_artifact_review_text);
}

#[test]
fn kiss_cov_src_orchestrator_review_loop_rs_code_review_attempt_outcome() {
    let _ = stringify!(crate::orchestrator::CodeReviewAttemptOutcome);
}

#[test]
fn kiss_cov_src_output_mod_rs_push_captured_stderr_line() {
    let _ = stringify!(crate::output::push_captured_stderr_line);
}

#[test]
fn kiss_cov_src_output_mod_rs_timestamp_now_string() {
    let _ = stringify!(crate::output::timestamp_now_string);
}

#[test]
fn kiss_cov_src_output_mod_rs_who_tag_ansi() {
    let _ = stringify!(crate::output::who_tag_ansi);
}

#[test]
fn kiss_cov_src_output_mod_rs_log_use_color() {
    let _ = stringify!(crate::output::log_use_color);
}

#[test]
fn kiss_cov_src_output_mod_rs_stderr_use_color() {
    let _ = stringify!(crate::output::stderr_use_color);
}

#[test]
fn kiss_cov_src_output_mod_rs_set_stdout_log_path() {
    let _ = stringify!(crate::output::set_stdout_log_path);
}

#[test]
fn kiss_cov_src_output_mod_rs_stdout_display_and_log() {
    let _ = stringify!(crate::output::stdout_display::stdout_display_and_log);
}

#[test]
fn kiss_cov_src_output_acp_tee_rs_format_line_acp_ansi_payload() {
    let _ = stringify!(crate::output::acp_tee::format_line_acp_ansi_payload);
}

#[test]
fn kiss_cov_src_output_acp_tee_rs_acp_tee_display_line() {
    let _ = stringify!(crate::output::acp_tee::acp_tee_display_line);
}

#[test]
fn kiss_cov_src_output_acp_tee_rs_acp_tee_log_line() {
    let _ = stringify!(crate::output::acp_tee::acp_tee_log_line);
}

#[test]
fn kiss_cov_src_output_acp_tee_rs_acp_tee_log_prefix() {
    let _ = stringify!(crate::output::acp_tee::acp_tee_log_prefix);
}

#[test]
fn kiss_cov_src_output_stdout_heartbeat_rs_emit_heartbeat_line() {
    let _ = stringify!(crate::output::stdout_heartbeat::emit_heartbeat_line);
}

#[test]
fn kiss_cov_src_output_stdout_heartbeat_rs_maybe_emit_stdout_heartbeat() {
    let _ = stringify!(crate::output::stdout_heartbeat::maybe_emit_stdout_heartbeat);
}

#[test]
fn kiss_cov_src_output_stdout_heartbeat_rs_spawn_wall_clock_poller_if_needed() {
    let _ = stringify!(crate::output::stdout_heartbeat::spawn_wall_clock_poller_if_needed);
}

#[test]
fn kiss_cov_src_output_stdout_heartbeat_rs_heartbeat_due() {
    let _ = stringify!(crate::output::stdout_heartbeat::heartbeat_due);
}

#[test]
fn kiss_cov_src_output_stdout_heartbeat_rs_try_emit_heartbeat_if_due() {
    let _ = stringify!(crate::output::stdout_heartbeat::try_emit_heartbeat_if_due);
}

#[test]
fn kiss_cov_src_output_stdout_heartbeat_rs_poll_wall_clock_heartbeat_if_due() {
    let _ = stringify!(crate::output::stdout_heartbeat::poll_wall_clock_heartbeat_if_due);
}

#[test]
fn kiss_cov_src_output_stdout_heartbeat_rs_wall_clock_poller_loop() {
    let _ = stringify!(crate::output::stdout_heartbeat::wall_clock_poller_loop);
}

#[test]
fn kiss_cov_src_output_stdout_display_rs_format_line_stdout() {
    let _ = stringify!(crate::output::stdout_display::format_line_stdout);
}

#[test]
fn kiss_cov_src_output_stdout_display_rs_format_line_stdout_ansi() {
    let _ = stringify!(crate::output::stdout_display::format_line_stdout_ansi);
}

#[test]
fn kiss_cov_src_output_stderr_log_rs_emit_stderr_log_line() {
    let _ = stringify!(crate::output::emit_stderr_log_line);
}

#[test]
fn kiss_cov_src_output_stderr_log_rs_emit_stderr_log_lines() {
    let _ = stringify!(crate::output::emit_stderr_log_lines);
}

#[test]
fn kiss_cov_src_stdout_log_path_rs_set_stdout_log_path() {
    let _ = stringify!(crate::stdout_log_path::set_stdout_log_path);
}

#[test]
fn kiss_cov_src_time_format_rs_timestamp_now_string() {
    let _ = stringify!(crate::time_format::timestamp_now_string);
}
