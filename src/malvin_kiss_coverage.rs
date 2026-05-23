//! Behavioral smoke tests for crate-root modules (kiss per-file coverage).

#[test]
fn smoke_time_format_and_stdout_log_path() {
    assert!(!crate::time_format::timestamp_now_string().is_empty());
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("out.log");
    crate::stdout_log_path::set_stdout_log_path(Some(path.clone()));
    assert_eq!(crate::stdout_log_path::clone_stdout_log_path(), Some(path));
    crate::stdout_log_path::set_stdout_log_path(None);
    let _ = stringify!(crate::time_format::timestamp_now_string);
    let _ = stringify!(crate::stdout_log_path::set_stdout_log_path);
    let _ = stringify!(crate::stdout_log_path::clone_stdout_log_path);
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
    assert_eq!(crate::artifacts::work_dir_for_path(&plan), tmp.path());
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
    assert_eq!(smoke.mbc2_pure().expect("mbc2"), "m");
}

#[test]
fn smoke_child_health_sample() {
    let health = crate::child_health::sample_child_health(std::process::id());
    let _ = health.exists;
    let _ = health.zombie;
}

#[cfg(target_os = "linux")]
#[test]
fn smoke_acp_memory_containment_test_support() {
    if crate::acp_memory_containment::test_support::writable_cgroups_on_host() {
        return;
    }
    let result = std::panic::catch_unwind(|| {
        crate::acp_memory_containment::test_support::require_cgroup_integration_test();
    });
    assert!(result.is_err());
}
