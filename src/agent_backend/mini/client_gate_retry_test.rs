use super::*;
use crate::agent_backend::mini::LoopDriverConfig;

#[test]
fn kiss_witness_gate_attempt_run_and_stop_check() {
    let config = LoopDriverConfig {
        max_http_turns: 4,
        max_bash_execs: 128,
        max_http_retries: 1,
        max_transport_retries: 3,
        max_shrink_passes: 0,
        mini_constraints: "c",
        expects_investigation: false,
    };
    let run = GateAttemptRun {
        prompt: "p",
        driver_config: &config,
        llm_phase: None,
        single_attempt: false,
        attempt: 1,
    };
    let GateAttemptRun {
        prompt,
        attempt,
        ..
    } = run;
    assert_eq!(prompt, "p");
    assert_eq!(attempt, 1);
    let stop = GateRetryStopCheck {
        single_attempt: true,
        timing: None,
        last_error: "e",
        attempt: 1,
        max_attempts: 2,
    };
    let GateRetryStopCheck {
        last_error,
        max_attempts,
        ..
    } = stop;
    assert_eq!(last_error, "e");
    assert_eq!(max_attempts, 2);
}

#[tokio::test]
async fn gate_retry_stop_single_attempt_returns_true() {
    let stop = should_stop_gate_retries(GateRetryStopCheck {
        single_attempt: true,
        timing: None,
        last_error: "fail",
        attempt: 1,
        max_attempts: 3,
    })
    .await
    .expect("stop check");
    assert!(stop);
}

#[tokio::test]
async fn gate_retry_stop_multi_attempt_continues_before_max() {
    let stop = should_stop_gate_retries(GateRetryStopCheck {
        single_attempt: false,
        timing: None,
        last_error: "fail",
        attempt: 1,
        max_attempts: 2,
    })
    .await
    .expect("stop check");
    assert!(!stop);
}

#[tokio::test]
async fn gate_retry_billing_failure_fails_fast_without_gate_attempt_message() {
    use crate::acp::CoderPromptOptions;
    use crate::agent_backend::mini::{LlmBackend, MiniAgentClient, MockScript, MockStep};
    use crate::agent_backend::test_support::{mini_loop_config, test_io};
    use crate::output::STDOUT_LOG_TEST_LOCK;
    use malvin_mini::OpenRouterError;

    let billing_msg = OpenRouterError::BillingFailure {
        status: 402,
        body: "no credits".into(),
    }
    .to_string();
    let llm = LlmBackend::Mock(std::sync::Mutex::new(MockScript {
        responses: vec![MockStep::BillingFailure {
            status: 402,
            body: "no credits".into(),
        }],
        call_count: 0,
        on_response: None,
    }));
    let mut config = mini_loop_config(1, 5);
    config.max_gate_retries = 5;
    let mut client = MiniAgentClient::new_mock(config, test_io(), llm);
    let work_dir = tempfile::tempdir().expect("tempdir");
    let log_path = work_dir.path().join("billing_gate.log");
    let _guard = STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    crate::output::clear_captured_stderr_lines();
    client.begin_coder_session(work_dir.path()).await.expect("begin");
    let err = client
        .run_coder_prompt(
            "task",
            &log_path,
            "billing_gate",
            CoderPromptOptions::default(),
        )
        .await
        .expect_err("billing must fail");
    assert!(
        err.0
            .to_ascii_lowercase()
            .contains("openrouter billing/credit failure"),
        "unexpected error: {}",
        err.0
    );
    assert!(
        !err.0.contains("mini gate attempt 1 failed"),
        "gate retry must not wrap billing: {}",
        err.0
    );
    let LlmBackend::Mock(m) = &client.llm else {
        panic!("mock llm");
    };
    assert_eq!(m.lock().expect("lock").call_count, 1, "single gate attempt");
    let _ = billing_msg;
}

#[tokio::test]
async fn gate_retry_stop_at_max_attempts_returns_true() {
    let stop = should_stop_gate_retries(GateRetryStopCheck {
        single_attempt: false,
        timing: None,
        last_error: "exhausted",
        attempt: 2,
        max_attempts: 2,
    })
    .await
    .expect("stop check");
    assert!(stop);
}
