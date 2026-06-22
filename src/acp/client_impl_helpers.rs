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
    match plan_agent_retry(last_error, attempt, max_attempts) {
        Err(e) => Err(e),
        Ok(AgentRetryOutcome::StopRetrying) => Ok(true),
        Ok(AgentRetryOutcome::Sleep(d)) => {
            crate::output::print_log_error(&format!(
                "agent acp attempt {attempt} failed: {last_error}"
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
#[path = "client_impl_helpers_kiss_cov_test.rs"]
mod client_impl_helpers_kiss_cov_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _ = teardown_coder_session_after_transport_error;
    }
}
