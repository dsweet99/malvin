use crate::acp::backoff_after_agent_failure;
use crate::test_stderr_capture::capture_stderr_output;

#[test]
fn backoff_does_not_log_when_retry_policy_stops_immediately() {
    let msg = "KPOP block prompt made no progress on the experiment log.";
    let stderr = capture_stderr_output(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        let stop = rt
            .block_on(backoff_after_agent_failure(None, msg, 1, 3))
            .expect("backoff");
        assert!(stop);
    });
    assert!(
        !stderr.contains("agent acp attempt"),
        "no-progress must not log at backoff when not retrying; stderr={stderr:?}"
    );
}

#[test]
fn backoff_logs_before_sleep_when_retry_will_occur() {
    let stderr = capture_stderr_output(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        let stop = rt
            .block_on(backoff_after_agent_failure(None, "request timed out", 1, 3))
            .expect("backoff");
        assert!(!stop);
    });
    assert!(
        stderr.contains("agent acp attempt 1 failed"),
        "retriable failure should log once before sleep; stderr={stderr:?}"
    );
}
