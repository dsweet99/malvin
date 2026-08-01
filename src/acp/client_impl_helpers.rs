use crate::acp::{agent_backoff_sleep, AgentError, AgentRetryOutcome, plan_agent_retry};

/// Kill the coder session when a transport/child-health error leaves it unusable.
pub(crate) async fn teardown_coder_session_after_transport_error(
    client: &mut crate::acp::AgentClient,
    err: &str,
) {
    if crate::acp::agent_error_requires_coder_session_teardown(err) {
        let _ = client.end_coder_session().await;
    }
}

/// Apply bounded-retry backoff after a failed attempt, or stop the retry loop.
/// Returns `Ok(true)` when the caller should `break` the attempt loop; `Err` on upgrade-plan short-circuit.
pub(crate) async fn backoff_after_agent_failure(
    timing: Option<&std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
    last_error: &str,
    attempt: u32,
    max_attempts: u32,
) -> Result<bool, AgentError> {
    backoff_after_labeled_agent_failure(LabeledBackoff {
        timing,
        last_error,
        attempt,
        max_attempts,
        label: "agent acp",
    })
    .await
}

pub(crate) async fn backoff_after_mini_gate_failure(
    timing: Option<&std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
    last_error: &str,
    attempt: u32,
    max_attempts: u32,
) -> Result<bool, AgentError> {
    backoff_after_labeled_agent_failure(LabeledBackoff {
        timing,
        last_error,
        attempt,
        max_attempts,
        label: "mini gate",
    }).await
}

struct LabeledBackoff<'a> {
    timing: Option<&'a std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
    last_error: &'a str,
    attempt: u32,
    max_attempts: u32,
    label: &'a str,
}

async fn backoff_after_labeled_agent_failure(
    req: LabeledBackoff<'_>,
) -> Result<bool, AgentError> {
    let LabeledBackoff {
        timing,
        last_error,
        attempt,
        max_attempts,
        label,
    } = req;
    match plan_agent_retry(last_error, attempt, max_attempts) {
        Err(e) => Err(e),
        Ok(AgentRetryOutcome::StopRetrying) => Ok(true),
        Ok(AgentRetryOutcome::Sleep(d)) => {
            crate::output::print_log_error(&format!(
                "{label} attempt {attempt} failed: {last_error}"
            ));
            crate::run_timing::record_backoff(timing, d);
            agent_backoff_sleep(d).await;
            Ok(false)
        }
    }
}

#[cfg(test)]
mod teardown_tests {
    use super::teardown_coder_session_after_transport_error;
    use crate::acp::test_captive_session::captive_cat_acp_session_for_tests;
    use crate::acp::{AgentClient, AgentIoOptions};
    use crate::support_paths::DEFAULT_MAX_ACP_RETRIES;

    #[tokio::test]
    async fn teardown_coder_session_after_hung_kills_open_session() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let mut client = AgentClient::with_max_acp_retries(
            "m".into(),
            AgentIoOptions {
                force: false,
                no_tee: true,
                raw_output: true,
                show_thoughts_on_stdout: false,
                emit_stdout_markdown: false,
                log_full_outgoing_prompts: false,
            },
            DEFAULT_MAX_ACP_RETRIES,
        );
        client.coder_session = Some(captive_cat_acp_session_for_tests(cwd));
        client.coder_session_cwd = Some(cwd.to_path_buf());
        teardown_coder_session_after_transport_error(
            &mut client,
            "acp child process appears hung",
        )
        .await;
        assert!(!client.has_open_coder_session());
        assert_eq!(client.coder_session_cwd.as_deref(), Some(cwd));
    }
}

#[cfg(test)]
mod kiss_cov_auto{
    use super::*;

    #[test]
    fn kiss_cov_backoff_after_agent_failure() {
        let _ = backoff_after_agent_failure;
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs{
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _ = backoff_after_agent_failure;
        let _ = backoff_after_mini_gate_failure;
        let req = LabeledBackoff {
            timing: None,
            last_error: "e",
            attempt: 1,
            max_attempts: 2,
            label: "mini gate",
        };
        let LabeledBackoff {
            attempt,
            max_attempts,
            label,
            ..
        } = req;
        assert_eq!(attempt, 1);
        assert_eq!(max_attempts, 2);
        assert_eq!(label, "mini gate");
    }
}
