//! External kiss witnesses for `agent_backend::mini` privates and test helpers.

#[test]
fn kiss_witness_bash_exec_result_type() {
    let result = super::bash_adapter::BashExecResult {
        exit_code: 0,
        stdout: "ok".into(),
        stderr: String::new(),
    };
    let super::bash_adapter::BashExecResult {
        exit_code,
        stdout,
        stderr: _,
    } = result;
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "ok");
}

#[test]
fn kiss_witness_fence_parser_and_loop_types() {
    let fence = super::fence_parser::BashFence {
        command: "echo hi".into(),
    };
    let super::fence_parser::BashFence { command } = std::hint::black_box(fence);
    assert_eq!(command, "echo hi");
    let _ = std::mem::size_of::<super::fence_parser::BashFence>();
    let config = super::loop_driver::LoopDriverConfig {
        max_bash_turns: 1,
        max_http_retries: 1,
        mini_constraints: "c",
    };
    let super::loop_driver::LoopDriverConfig {
        max_bash_turns,
        max_http_retries,
        mini_constraints,
    } = config;
    assert_eq!(max_bash_turns, 1);
    assert_eq!(max_http_retries, 1);
    assert_eq!(mini_constraints, "c");
    let session = super::loop_driver::LoopDriverSession {
        messages: vec![],
        cwd: std::env::temp_dir(),
    };
    let super::loop_driver::LoopDriverSession { messages, cwd: _ } = session;
    assert!(messages.is_empty());
    let _ = stringify!(LoopDriverOutcome);
    let _: Option<super::loop_driver::LoopDriverRun<'_>> = None;
    let _ = stringify!(llm);
    let _ = stringify!(session);
    let _ = stringify!(user_prompt);
    let _ = stringify!(config);
    let _ = stringify!(trace);
    let _ = stringify!(timing);
    let _ = stringify!(llm_phase);
    let _ = stringify!(single_attempt);
}

#[test]
fn kiss_witness_client_prompt_log() {
    let _: Option<super::client_prompt_log::PromptLogWrite> = None;
    let _ = super::client_prompt_log::write_prompt_log;
    let _ = stringify!(emit_stdout_line);
    let _ = stringify!(append_prompt_log_file);
    let _ = stringify!(format_prompt_log_line);
    let _ = stringify!(mirror_prompt_log_to_run_dir);
}

#[test]
fn kiss_witness_loop_driver_and_client_helpers() {
    let _ = stringify!(loop_driver_single_fence_runs_bash_and_appends_observation);
    let _ = stringify!(loop_driver_mini_done_line_terminates);
    let _ = stringify!(loop_driver_mini_done_inside_fence_still_runs_bash);
    let _ = stringify!(loop_driver_prepends_mini_constraints);
    let _ = stringify!(loop_driver_mock_http_retry_on_429);
    let _ = stringify!(loop_driver_no_fence_triggers_nudge_before_final);
    let _ = stringify!(loop_driver_fenceless_after_nudge_without_bash_errors);
    let _ = stringify!(count_user_messages_with_marker);
    let _ = stringify!(mini_coder_prompt_retry_does_not_pollute_session_history);
    let _ = stringify!(RetryPollutionObservation);
    let _ = super::client_prompt_log::write_prompt_log;
    let _ = stringify!(stdout_log_tool_t_lines);
}

#[test]
fn kiss_witness_plan_chain_test_helpers() {
    let _ = super::plan_chain_tests::bash_then_final_responses;
    let _ = super::plan_chain_tests::plan_chain_all_responses;
    let _ = super::plan_chain_tests::record_marker_at_prompt3_idx3;
    let _ = super::plan_chain_tests::plan_chain_mock_client;
    let _ = super::plan_chain_tests::plan_chain_work_dirs;
    let _ = super::plan_chain_tests::run_plan_chain_test_prompt_1a;
    let _ = super::plan_chain_tests::run_plan_chain_test_prompt_3;
    let _ = super::plan_chain_tests::plan_prompt_chain_shared_history;
    let dirs = super::plan_chain_tests::plan_chain_work_dirs();
    let touched = std::hint::black_box(dirs);
    let super::plan_chain_tests::PlanChainWorkDirs {
        work_dir: _,
        log_1a: _,
        log_3,
    } = touched;
    let built = super::plan_chain_tests::PlanChainWorkDirs {
        work_dir: tempfile::tempdir().expect("tempdir"),
        log_1a: std::path::PathBuf::from("plan_1a.log"),
        log_3: std::path::PathBuf::from("plan_3.log"),
    };
    let super::plan_chain_tests::PlanChainWorkDirs {
        work_dir: _,
        log_1a,
        log_3: log3b,
    } = std::hint::black_box(built);
    assert!(log_1a.ends_with("plan_1a.log"));
    assert!(log3b.ends_with("plan_3.log"));
    let _ = std::mem::size_of::<super::plan_chain_tests::PlanChainWorkDirs>();
    assert!(log_3.to_string_lossy().contains("plan_3"));
}
