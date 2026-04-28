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
    let text = msg.to_ascii_lowercase();
    if text.contains("timed out")
        || timeout_word_without_identifier_false_positive(&text)
        || text.contains("deadline exceeded")
        || text.contains("deadlineexceeded")
    {
        return true;
    }
    if text.contains("writableiterable is closed") || text.contains("readableiterable is closed") {
        return true;
    }
    if text.contains("child process is dead")
        || text.contains("child process is zombie")
        || text.contains("dead or zombie child process")
        || text.contains("child process is not running")
    {
        return true;
    }
    if has_delimited_substring(&text, "initialize session")
        || has_delimited_substring(&text, "session initialization")
        || has_delimited_substring(&text, "session/new")
        || has_delimited_substring(&text, "session init")
    {
        return true;
    }
    text.contains("[unavailable]")
}

fn timeout_word_without_identifier_false_positive(text: &str) -> bool {
    let needle = "timeout";
    let mut search_from = 0_usize;
    while let Some(found) = text[search_from..].find(needle) {
        let start = search_from + found;
        let end = start + needle.len();
        let before = if start == 0 {
            b' '
        } else {
            text.as_bytes()[start - 1]
        };
        let after = if end >= text.len() {
            b' '
        } else {
            text.as_bytes()[end]
        };
        if !is_identifier_byte(before) && !is_identifier_byte(after) {
            return true;
        }
        search_from = end;
    }
    false
}

const fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-'
}

fn has_delimited_substring(text: &str, token: &str) -> bool {
    let mut search_from = 0_usize;
    while let Some(found) = text[search_from..].find(token) {
        let start = search_from + found;
        let end = start + token.len();
        let before = if start == 0 {
            b' '
        } else {
            text.as_bytes()[start - 1]
        };
        let after = if end >= text.len() {
            b' '
        } else {
            text.as_bytes()[end]
        };
        if !is_identifier_byte(before) && !is_identifier_byte(after) {
            return true;
        }
        search_from = end;
    }
    false
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
    if !agent_string_is_retriable(last_error) || attempt >= MAX_AGENT_ATTEMPTS {
        return Ok(AgentRetryOutcome::StopRetrying);
    }
    let secs = if attempt == 1 { 1_u64 } else { 3_u64 };
    Ok(AgentRetryOutcome::Sleep(std::time::Duration::from_secs(secs)))
}
