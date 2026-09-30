use crate::acp::{AgentError, AttemptCeiling, backoff_after_agent_failure};
use crate::test_stderr_capture::capture_stderr_output;

#[test]
fn backoff_does_not_log_when_retry_policy_stops_immediately() {
    let stderr = capture_stderr_output(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        let stop = rt
            .block_on(backoff_after_agent_failure(
                None,
                &AgentError::ordinary("timed out"),
                3,
                AttemptCeiling::at_most_u32(3),
            ))
            .expect("backoff");
        assert!(stop);
    });
    assert!(
        !stderr.contains("agent attempt"),
        "exhausted retries must not log at backoff; stderr={stderr:?}"
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
            .block_on(backoff_after_agent_failure(
                None,
                &AgentError::ordinary("request timed out"),
                1,
                AttemptCeiling::at_most_u32(3),
            ))
            .expect("backoff");
        assert!(!stop);
    });
    assert!(
        stderr.contains("agent attempt 1 failed"),
        "retriable failure should log once before sleep; stderr={stderr:?}"
    );
}
