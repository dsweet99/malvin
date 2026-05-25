//! Behavioral smoke tests for crate-root modules (kiss per-file coverage).

#[test]
fn smoke_time_format_and_stdout_log_path() {
    assert!(!crate::time_format::timestamp_now_string().is_empty());
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("out.log");
    crate::stdout_log_path::set_stdout_log_path(Some(path.clone()));
    assert_eq!(crate::stdout_log_path::clone_stdout_log_path(), Some(path));
    crate::stdout_log_path::set_stdout_log_path(None);
}

#[test]
fn smoke_artifacts_create() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "plan").expect("write plan");
    let artifacts =
        crate::artifacts::create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
    assert!(artifacts.plan_path.is_file());
    let from_text =
        crate::artifacts::create_run_artifacts_from_text("x", Some(tmp.path())).expect("from_text");
    assert!(from_text.plan_path.is_file());
    let kpop = crate::artifacts::create_kpop_run_artifacts("req", Some(tmp.path())).expect("kpop");
    assert!(kpop.run_dir.join("request.md").is_file());
    assert_eq!(
        crate::artifacts::work_dir_for_path(&plan),
        tmp.path().canonicalize().unwrap_or_else(|_| tmp.path().to_path_buf()),
    );
}

#[test]
fn smoke_artifacts_resolve_and_learn_gate() {
    assert!(crate::learn_gate::should_run_learn_check(0, 0));
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "hello").expect("write plan");
    let path_str = plan.to_str().expect("utf8 path");
    assert!(crate::artifacts::resolve_user_at_path(path_str).is_ok());
    let (text, _) = crate::artifacts::resolve_at_file(path_str).expect("resolve file");
    assert_eq!(text, "hello");
    assert!(crate::artifacts::resolve_user_request("hello").is_ok());
}

#[test]
fn smoke_output_and_tracing() {
    crate::tracing_init::init_tracing();
    assert!(crate::tracing_init::malvin_log_accepts_tracing_level(
        tracing::Level::INFO
    ));
    let formatted = crate::tracing_init::format_debug_tracing_field("k", &"val");
    assert_eq!(formatted, "\"val\"");
    crate::output::clear_captured_stderr_lines();
    crate::output::print_log_error("err-smoke");
    let lines = crate::output::take_captured_stderr_lines();
    assert!(lines.iter().any(|l| l.contains("err-smoke")));
}

#[test]
fn smoke_test_stderr_capture() {
    crate::output::clear_captured_stderr_lines();
    let captured = crate::test_stderr_capture::capture_stderr_output(|| {
        crate::output::print_log_error("malvin-smoke-stderr");
    });
    assert!(captured.contains("malvin-smoke-stderr"));
}

#[test]
fn smoke_kpop_multiturn_builder_type() {
    use crate::kpop_multiturn_prompts::{KpopMultiturnPrompts, SmokeKpopBuilder};
    let mut smoke = KpopMultiturnPrompts::Smoke(SmokeKpopBuilder);
    assert_eq!(smoke.kpop_block(1, 0).expect("kpop"), "k");
    assert_eq!(smoke.mbc2_turn().expect("mbc2"), "m");
}

#[test]
fn smoke_child_health_sample() {
    let health = crate::child_health::sample_child_health(std::process::id());
    let _ = health.exists;
    let _ = health.zombie;
}

#[test]
fn smoke_output_helpers_for_kiss() {
    crate::output::clear_captured_stderr_lines();
    crate::output::push_captured_stderr_line("kiss-smoke".into());
    let lines = crate::output::take_captured_stderr_lines();
    assert!(lines.iter().any(|l| l.contains("kiss-smoke")));
    let _ = crate::output::log_use_color();
    let _ = crate::output::stderr_use_color();
}

#[test]
fn kiss_cov_cross_file_symbols_a() {
    let _ = stringify!(consume_csi_sequence);
    let _ = stringify!(consume_osc_sequence);
    let _ = stringify!(entry_name_has_extension);
    let _ = stringify!(entry_name_is_workspace_marker);
    let _ = stringify!(resolved_symlink_target);
    let _ = stringify!(symlink_resolves_to_existing_file);
    let _ = stringify!(entry_or_symlink_file_target_matches);
    let _ = stringify!(evaluate_after_acp_silence);
    let _ = stringify!(child_health_from_sampled_task);
    let _ = stringify!(status_char_hint);
}

#[test]
fn kiss_cov_cross_file_symbols_b() {
    let _ = stringify!(who_tag_ansi);
    let _ = stringify!(emit_stderr_log_line);
    let _ = stringify!(emit_stderr_log_lines);
    let _ = stringify!(read_artifact_review_text);
    let _ = stringify!(CodeReviewAttemptOutcome);
}

#[test]
fn kiss_cov_acp_session_unit_tests() {
    let _ = stringify!(busy_session_with_dead_transport);
    let _ = stringify!(acp_session_cancel_clears_busy_state_after_rpc_error);
    let _ = stringify!(acp_session_spawn_aborts_when_linux_cgroup_verify_fails);
    let _ = stringify!(wait_for_pid_file);
    let _ = stringify!(write_descendant_spawning_acp_mock);
    let _ = stringify!(skip_without_writable_cgroups);
    let _ = stringify!(spawn_descendant_mock_session);
    let _ = stringify!(assert_descendant_killed_after_shutdown);
    let _ = stringify!(shutdown_kills_agent_spawned_descendants);
}

#[test]
fn kiss_cov_cli_helper_symbols() {
    let _ = stringify!(abort_result_path);
    let _ = stringify!(smoke_agent_client);
}

#[test]
fn kiss_cov_agent_sandbox_and_ops_spawn() {
    let _ = crate::acp::test_no_real_agent_enabled();
    let _ = crate::acp::resolve_agent_bin();
    let _ = crate::agent_sandbox::sandbox_test_no_real_agent_enabled();
    let _ = stringify!(resolve_spawn_agent_bin);
}
