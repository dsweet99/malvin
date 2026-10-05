use crate::agent_process::{AgentError, AttemptCeiling, backoff_after_agent_failure, retries_noun};

use super::backend_error_stop::{backend_error_stop, with_local_backend_hint};
use super::sdk_client::SdkClient;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BackoffChoice {
    Consult,
    StopWithoutSleep,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum StopDecision {
    Continue,
    Exhausted,
}

pub(super) struct RetrySpec<'a> {
    pub ceiling: AttemptCeiling,
    pub failure_lead: &'a str,
    pub backoff: BackoffChoice,
}

pub(crate) fn exhaustion_error(
    client: &SdkClient,
    lead: &str,
    retries: u32,
    last_error: &str,
) -> AgentError {
    let message = format!(
        "{lead} after {retries} {}. Last error:\n{last_error}",
        retries_noun(retries)
    );
    AgentError(with_local_backend_hint(client, message))
}

pub(crate) async fn stop_after_failure(
    client: &mut SdkClient,
    spec: &RetrySpec<'_>,
    err: &AgentError,
    attempts_used: u32,
) -> Result<StopDecision, AgentError> {
    if let Some(stop) = backend_error_stop(client, &err.message) {
        return Err(stop);
    }
    if spec.backoff == BackoffChoice::StopWithoutSleep {
        return Ok(StopDecision::Exhausted);
    }
    let stop =
        backoff_after_agent_failure(client.timing.as_ref(), err, attempts_used, spec.ceiling)
            .await?;
    Ok(if stop {
        StopDecision::Exhausted
    } else {
        StopDecision::Continue
    })
}

macro_rules! retry_until_ok {
    (
        $client:ident,
        $spec:expr,
        |$c_once:ident| $once:expr,
        |$c_err:ident, $err:ident| $on_err:expr
    ) => {{
        let spec = $spec;
        let mut attempts_used = 0_u32;
        let mut last_error = String::new();
        'acp_attempt_done: {
            loop {
                attempts_used = attempts_used.saturating_add(1);
                let $c_once = &mut *$client;
                match $once {
                    Ok(value) => break 'acp_attempt_done Ok(value),
                    Err(failed) => {
                        let $c_err = &mut *$client;
                        let $err = failed;
                        match $on_err {
                            Err(halt) => break 'acp_attempt_done Err(halt),
                            Ok(kept) => {
                                last_error.clone_from(&kept.message);
                                if $crate::backends::agent_backend::acp_attempt_loop::stop_after_failure(
                                    $client,
                                    &spec,
                                    &kept,
                                    attempts_used,
                                )
                                .await?
                                    == $crate::backends::agent_backend::acp_attempt_loop::StopDecision::Exhausted
                                {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            Err($crate::backends::agent_backend::acp_attempt_loop::exhaustion_error(
                $client,
                spec.failure_lead,
                attempts_used.saturating_sub(1),
                &last_error,
            ))
        }
    }};
}

pub(super) use retry_until_ok;
