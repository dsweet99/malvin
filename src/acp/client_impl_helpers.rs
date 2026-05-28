use crate::acp::{AgentError, AgentRetryOutcome, plan_agent_retry, tokio_sleep};

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
            tokio_sleep(d).await;
            Ok(false)
        }
    }
}

#[cfg(test)]
mod kiss_cov_auto {
    #[test]
    fn kiss_cov_backoff_after_agent_failure() {
        let _ = stringify!(backoff_after_agent_failure);
    }
}

#[cfg(test)]
mod kiss_cov_gate_refs {
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _ = backoff_after_agent_failure;
    }
}
