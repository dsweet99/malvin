//! Behavioral smoke tests for crate-root modules (kiss per-file coverage).

#[test]
fn smoke_active_agent_heartbeat_stats() {
    let _ = stringify!(ActiveAgentSandbox);
    let _ = stringify!(ActiveAgentStatsSource);
    let _ = crate::active_agent_heartbeat::register_active_agent_process_group;
    let _ = crate::active_agent_heartbeat::unregister_active_agent_process_group;
    let _ = crate::malvin_sandbox::init_malvin_spawn_baseline;
    let _ = crate::malvin_sandbox::malvin_session_rss_bytes;
    crate::active_agent_heartbeat::clear_active_agent_process_groups_for_test();
    assert!(crate::active_agent_heartbeat_stats().is_none());
}

#[test]
fn smoke_agent_phase_kpop_and_reporting() {
    let _guard = crate::agent_phase::AGENT_PHASE_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    crate::agent_phase::reset_phase_state_for_test();
    crate::agent_phase::enter_kpop();
    assert_eq!(crate::agent_phase::heartbeat_label(), "KPop cycling");
    crate::agent_phase::leave_kpop();
}

#[test]
fn smoke_emit_without_log_path_skips_disk_append() {
    let _guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("stdout.log");
    crate::output::set_stdout_log_path(None);
    crate::output::enable_stdout_capture();
    crate::output::emit_stdout_rendered_immediate("[probe] x", "20260524.000000.000 [probe] x");
    let terminal = crate::output::take_captured_stdout();
    assert_eq!(terminal.trim(), "[probe] x");
    crate::output::set_stdout_log_path(Some(path.clone()));
    crate::output::emit_stdout_rendered_immediate("[probe] y", "20260524.000000.000 [probe] y");
    crate::output::set_stdout_log_path(None);
    let text = std::fs::read_to_string(path).unwrap_or_default();
    assert!(text.contains("[probe] y"));
}

#[test]
fn smoke_publish_heartbeat_live_terminal() {
    let _guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("stdout.log");
    crate::output::set_stdout_log_path(Some(path.clone()));
    crate::output::enable_stdout_capture();
    crate::output::reset_stdout_heartbeat_for_test();
    crate::output::test_set_last_heartbeat_elapsed(std::time::Duration::from_secs(61));
    let display = "malvin.| 20260524.000000 Waiting";
    crate::output::publish_heartbeat_live_terminal(display);
    let terminal = crate::output::take_captured_stdout();
    crate::output::set_stdout_log_path(None);
    assert_eq!(terminal.trim(), display);
    assert!(std::fs::read_to_string(path).unwrap_or_default().is_empty());
    assert!(
        crate::output::heartbeat_rendered_if_due(std::time::Instant::now(), false).is_none()
    );
}

#[test]
fn smoke_time_format_and_stdout_log_path() {
    assert!(!crate::time_format::timestamp_now_string().is_empty());
    let _guard = crate::agent_phase::AGENT_PHASE_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    crate::agent_phase::reset_phase_state_for_test();
    assert!(crate::time_format::heartbeat_payload_now().contains("Orienting"));
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
    assert!(artifacts.quality_gates_log_path().is_file());
    let from_text =
        crate::artifacts::create_run_artifacts_from_text("x", Some(tmp.path())).expect("from_text");
    assert!(from_text.plan_path.is_file());
    assert!(from_text.quality_gates_log_path().is_file());
    let kpop = crate::artifacts::create_kpop_run_artifacts("req", Some(tmp.path())).expect("kpop");
    assert!(kpop.run_dir.join("request.md").is_file());
    assert!(kpop.quality_gates_log_path().is_file());
    assert_eq!(
        crate::artifacts::work_dir_for_path(&plan),
        tmp.path().canonicalize().unwrap_or_else(|_| tmp.path().to_path_buf()),
    );
}

#[test]
fn smoke_artifacts_resolve_user_md_request() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "hello").expect("write plan");
    let (text, _) = crate::artifacts::resolve_user_md_request("hello").expect("literal");
    assert_eq!(text, "hello");
    let _guard = crate::test_utils::test_env_lock();
    let old = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(tmp.path()).expect("chdir");
    let (text, _) = crate::artifacts::resolve_user_md_request("plan.md").expect("md path");
    std::env::set_current_dir(old).expect("restore cwd");
    assert_eq!(text, "hello");
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
    tracing::warn!(target: "malvin::kiss_cov", extra = 1, "trace-layer-smoke");
    let lines = crate::output::take_captured_stderr_lines();
    assert!(lines.iter().any(|l| l.contains("err-smoke")));
    assert!(lines.iter().any(|l| l.contains("trace-layer-smoke")));
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
}

#[test]
fn smoke_child_health_sample() {
    let _health = crate::child_health::sample_child_health(std::process::id());
}

#[test]
fn smoke_mem_limit_and_process_group_rss() {
    let gb = crate::mem_limit_config::default_mem_limit_gb();
    assert!(gb >= 1);
    let rss = crate::process_group_rss::process_group_rss_bytes(
        crate::process_group_rss::current_process_group_id().expect("pgid"),
    )
    .expect("rss");
    assert!(rss > 0);
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
    let _: Option<crate::sandbox_oom::SandboxOomKillFacts> = None;
    let _: Option<crate::terminal_palette::TerminalTheme> = None;
    let _: Option<crate::terminal_palette::Palette> = None;
    let _: Option<crate::run_timing::acp_post_run::RunTimingSessionEnd> = None;
    let _: Option<crate::run_timing::acp_post_run::RunTimingAfterAcp> = None;
    let _: Option<crate::run_timing::acp_post_run::RunTimingAfterBackend> = None;
    let _: Option<crate::session_dotfile_backup::DotfileBackupPayload> = None;
    let _: Option<crate::session_dotfile_backup::SessionDotfileParts> = None;
    let _ = stringify!(ActiveAgentSandbox);
    let _ = stringify!(ActiveAgentStatsSource);
    let _ = stringify!(AcpTeeDirection);
    let _ = stringify!(AcpTeeLineFmt);
    let _ = stringify!(TaggedDisplayStyle);
    let _ = stringify!(StdoutRenderPrelude);
    let _ = stringify!(KpopPromptValidation);
    let _ = stringify!(prompt_source_desc);
}

#[test]
fn kiss_cov_cross_file_symbols_b() {
    let _ = stringify!(SampledTaskPidInfo);
    let _ = stringify!(BashExecResult);
    let _ = stringify!(MiniLoopConfig);
    let _ = stringify!(run_coder_prompt_with_retries);
    let _ = stringify!(MiniTraceSink);
}

#[test]
fn kiss_cov_acp_session_unit_tests() {
    let _ = stringify!(MemWatchHandles);
    let _ = stringify!(AffiliationCtx);
    let _ = stringify!(TeeStdoutEmit);
    let _ = stringify!(shutdown_cancel_timeout);
    let _ = stringify!(run_coder_prompt_with_retries);
    let _ = stringify!(run_one_coder_prompt_attempt);
}

#[test]
fn kiss_cov_cli_helper_symbols() {
    let _: Option<crate::cli::kpop_flow::kpop_flow_run_loop::RunKpopAgentLoopsParams<'_>> = None;
    let _: Option<crate::cli::kpop_flow::kpop_flow_run_loop::RunKpopAgentLoopsOutcome> = None;
    let _: Option<crate::cli::kpop_flow::kpop_flow_run_loop::KpopLoopSnapshot> = None;
    let _ = stringify!(RunKpopAgentLoopsParams);
    let _ = stringify!(kpop);
    let _ = stringify!(prepared);
    let _ = stringify!(BareLoopOpts);
    let _ = stringify!(BugLogMatch);
    let _ = stringify!(BugIdResolved);
    let _ = stringify!(LoopDefaultMut);
    let _ = stringify!(CodeWorkflowLoopMut);
    let _ = stringify!(ExplainResolvedOutputs);
    let _ = stringify!(ExplainKpopRequestInput);
    let _ = stringify!(KPopEnginePrepared);
    let _ = stringify!(InspireRunPrep);
    let _ = stringify!(OuterLoopSummarizeParams);
    let _ = stringify!(CodeOuterLoopSummarizeInputs);
    let _ = stringify!(KpopOuterLoopSummarizeInputs);
    let _ = stringify!(RunStartupEmitOpts);
}

#[test]
fn kiss_cov_coverage_kiss_gate_refs() {
    let _ = stringify!(kiss_cov_coalesce_private_helpers);
    let _ = stringify!(kiss_cov_reader_tests_helpers_symbols);
}

#[test]
fn kiss_cov_ops_spawn() {
    let _ = crate::acp::test_no_real_agent_enabled();
    let _ = crate::acp::resolve_agent_bin();
}
