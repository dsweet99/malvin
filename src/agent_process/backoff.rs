use crate::agent_process::{
    AgentError, AgentRetryOutcome, AttemptCeiling, agent_backoff_sleep, plan_agent_retry,
};

pub(crate) async fn backoff_after_agent_failure(
    timing: Option<&std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
    err: &AgentError,
    attempt: u32,
    ceiling: AttemptCeiling,
) -> Result<bool, AgentError> {
    match plan_agent_retry(err, attempt, ceiling) {
        Err(e) => Err(e),
        Ok(AgentRetryOutcome::StopRetrying) => Ok(true),
        Ok(AgentRetryOutcome::Sleep(d)) => {
            crate::output::print_log_error(&format!(
                "agent attempt {attempt} failed: {}",
                err.message
            ));
            crate::run_timing::record_backoff(timing, d);
            agent_backoff_sleep(d).await;
            Ok(false)
        }
    }
}
