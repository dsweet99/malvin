// Bounded retries for transient ACP JSON-RPC failures (see product plan: up to 3 attempts, 1s / 3s backoff).
// Covers `session/prompt` and spawn/handshake (`initialize` / `authenticate` / `session/new`) via
// [`AgentClient::begin_coder_session`], which retries the full `agent acp` session setup.

pub(crate) const MAX_AGENT_ATTEMPTS: u32 = 3;

/// English noun for `n` retry attempts after the first try (`n` is `attempts_used - 1` in callers).
#[must_use]
pub(crate) const fn retries_noun(n: u32) -> &'static str {
    if n == 1 { "retry" } else { "retries" }
}

pub(crate) fn agent_string_is_upgrade_plan(msg: &str) -> bool {
    msg.to_ascii_lowercase()
        .contains("upgrade your plan to continue")
}

pub(crate) fn agent_string_is_retriable(msg: &str) -> bool {
    !agent_string_is_upgrade_plan(msg)
}

#[derive(Debug)]
pub(crate) enum AgentRetryOutcome {
    StopRetrying,
    Sleep(std::time::Duration),
}

/// Shared retry policy for bounded ACP attempts (upgrade-plan / funds-exceeded errors fail fast;
/// everything else retries with 1s then 3s sleeps).
pub(crate) fn plan_agent_retry(last_error: &str, attempt: u32) -> Result<AgentRetryOutcome, AgentError> {
    if agent_string_is_upgrade_plan(last_error) {
        return Err(AgentError(last_error.to_string()));
    }
    if !agent_string_is_retriable(last_error) || attempt == MAX_AGENT_ATTEMPTS {
        return Ok(AgentRetryOutcome::StopRetrying);
    }
    let secs = if attempt == 1 { 1_u64 } else { 3_u64 };
    Ok(AgentRetryOutcome::Sleep(std::time::Duration::from_secs(secs)))
}
