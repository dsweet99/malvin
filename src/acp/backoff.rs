
use crate::acp::{agent_backoff_sleep, AgentError, AgentRetryOutcome, plan_agent_retry};

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
                "agent attempt {attempt} failed: {last_error}"
            ));
            crate::run_timing::record_backoff(timing, d);
            agent_backoff_sleep(d).await;
            Ok(false)
        }
    }
}
