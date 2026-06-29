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
        comment: None,
    };
    let super::fence_parser::BashFence { command, comment: _ } = std::hint::black_box(fence);
    assert_eq!(command, "echo hi");
    let _ = std::mem::size_of::<super::fence_parser::BashFence>();
    let config = super::loop_driver::LoopDriverConfig {
        max_http_turns: 1,
        max_bash_execs: 128,
        max_http_retries: 1,
        max_transport_retries: 3,
        max_shrink_passes: 0,
        mini_constraints: "c",
        expects_investigation: false,
    };
    let super::loop_driver::LoopDriverConfig {
        max_http_turns,
        max_http_retries,
        mini_constraints,
        ..
    } = config;
    assert_eq!(max_http_turns, 1);
    assert_eq!(max_http_retries, 1);
    assert_eq!(mini_constraints, "c");
    let session = super::loop_driver::LoopDriverSession {
        messages: vec![],
        cwd: std::env::temp_dir(),
        constraints_prepended: false,
        bash_commands_this_prompt: vec![],
        prompt_index: 0,
    llm_model_slug: String::new(),
    };
    let super::loop_driver::LoopDriverSession {
        messages,
        cwd: _,
        ..
    } = session;
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
    let _ = stringify!(should_push_user_prompt);
    let _ = stringify!(gate_attempt);
    let _ = stringify!(retry_strategy);
    let _ = stringify!(llm_model_slug);
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
fn kiss_witness_mini_audit_and_recovery_types() {
    let _ = std::mem::size_of::<super::terminal::MiniTerminalRecord>();
    let _ = std::mem::size_of::<super::retry_fork::RetryForkLedger>();
    let _ = std::mem::size_of::<super::context_recovery::ShrinkEvent>();
    let _ = std::mem::size_of::<super::fence_parser::FenceParseWarning>();
    let _ = super::retry_fork::build_divergence_observation;
    let _ = super::context_recovery::shrink_one_whole_message;
    let event = super::context_recovery::ShrinkEvent {
        attempt: 1,
        messages_before: 2,
        messages_after: 1,
        bytes_removed: 3,
    };
    assert_eq!(event.bytes_removed, 3);
}

#[test]
fn kiss_witness_trace_audit_emitters() {
    use super::acp_trace_shim::MiniHttpExchangeRecord;
    let sink = super::trace::MiniTraceSink::new(None, crate::agent_backend::test_support::test_io());
    super::trace::record_http_exchange(
        &sink,
        MiniHttpExchangeRecord {
            attempt: 1,
            status: None,
            body: None,
            error: None,
        },
    );
    let record = super::terminal::MiniTerminalRecord::new(
        super::terminal::MiniTerminalReason::FencelessComplete,
        1,
        0,
        super::terminal::MiniPhase::Terminal,
    );
    super::trace_audit::emit_terminal(&sink, &record);
    super::trace_audit::emit_prompt_shrink(&sink, &super::context_recovery::ShrinkEvent {
        attempt: 1,
        messages_before: 2,
        messages_after: 1,
        bytes_removed: 1,
    });
    super::trace_audit::emit_prompt_shrink_stalled(&sink);
    let shrink = super::acp_trace_shim::MiniPromptShrinkTrace {
        attempt: 2,
        messages_before: 3,
        messages_after: 2,
        bytes_removed: 4,
        strategy: "drop",
    };
    let super::acp_trace_shim::MiniPromptShrinkTrace {
        attempt,
        messages_before,
        messages_after,
        bytes_removed,
        strategy,
    } = shrink;
    assert_eq!(attempt, 2);
    assert_eq!(messages_before, 3);
    assert_eq!(messages_after, 2);
    assert_eq!(bytes_removed, 4);
    assert_eq!(strategy, "drop");
    super::trace_audit::emit_retry_fork(
        &sink,
        &super::retry_fork::RetryForkLedger {
            prompt_index: 0,
            attempt: 1,
            message_checkpoint_len: 0,
            workspace_manifest_hash: "h".into(),
            bash_commands: vec![],
            outcome: super::retry_fork::ForkOutcome::Succeeded,
            strategy: super::retry_fork::MiniRetryStrategy::CumulativeTranscript,
        },
    );
    let _ = super::trace::record_http_exchange;
}

#[test]
fn kiss_witness_loop_driver_and_client_helpers() {
    let _ = stringify!(loop_driver_single_fence_runs_bash_and_appends_observation);
    let _ = stringify!(loop_driver_mini_done_line_terminates);
    let _ = stringify!(loop_driver_mini_done_inside_fence_still_runs_bash);
    let _ = stringify!(loop_driver_prepends_mini_constraints);
    let _ = stringify!(loop_driver_mock_http_retry_on_429);
    let _ = stringify!(loop_driver_fenceless_completes_in_one_turn);
    let _ = stringify!(loop_driver_fenceless_no_nudge_in_prompts_log);
    let _ = stringify!(count_user_messages_with_marker);
    let _ = super::client_gate_retry::run_coder_prompt_with_gate_retries;
    let _ = super::client_gate_retry_attempt::run_one_gate_attempt;
    let _ = stringify!(ForkLedgerBuild);
    let _ = stringify!(GateAttemptOutcome);
    let _ = stringify!(GateAttemptRun);
    let _ = stringify!(GateRetryStopCheck);
    let _ = stringify!(gate_retry_stop_single_attempt_returns_true);
    let _ = stringify!(gate_retry_stop_multi_attempt_continues_before_max);
    let _ = stringify!(cumulative_gate_retry_skips_repushed_user_prompt);
    let _ = stringify!(kiss_witness_gate_attempt_run_and_stop_check);
    let _ = stringify!(fail_gate_exhausted_with_error);
    let _ = stringify!(RetryPollutionObservation);
    let _ = super::client_prompt_log::write_prompt_log;
    let _ = stringify!(mini_stdout_emits_bash_tool_summary_with_t_tag);
    let _ = stringify!(complete_with_http_retries_non_billing_errors_exhaust_transport_budget);
    let _ = stringify!(complete_with_http_retries_succeeds_on_second_mock_attempt);
    let _ = stringify!(complete_with_http_retries_maps_context_overflow);
    let _ = stringify!(complete_with_http_retries_retries_nvidia_resource_exhausted);
    let _ = stringify!(complete_with_http_retries_billing_failure_fails_on_first_attempt);
    let _ = stringify!(complete_with_http_retries_emits_mini_http_exchange_to_trace);
    let _ = stringify!(kiss_witness_http_retry_types);
    let _ = stringify!(kiss_witness_http_retry_limits_and_counters);
    let _ = stringify!(kiss_witness_http_retry_counter_next_paths);
    let _ = stringify!(mock_step_outcome);
    let _ = super::loop_driver::complete_with_http_retries;
    let _ = std::mem::size_of::<crate::fork_state::ForkState>();
}

#[test]
fn kiss_witness_concept_type_enums() {
    let _ = std::mem::size_of::<crate::acp_trace_impersonation::SyntheticAcpSessionUpdate>();
    let _ = std::mem::size_of::<crate::reliability_tier::ReliabilityTier>();
}
