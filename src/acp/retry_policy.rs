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

pub(crate) fn agent_string_is_cannot_use_model(msg: &str) -> bool {
    msg.to_ascii_lowercase().contains("cannot use this model")
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
    delimited_token_match(text, "timeout")
}

const fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-'
}

fn has_delimited_substring(text: &str, token: &str) -> bool {
    delimited_token_match(text, token)
}

fn delimited_token_match(text: &str, token: &str) -> bool {
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

/// Shared retry policy for bounded ACP attempts (upgrade-plan / invalid-model errors fail fast;
/// everything else retries with 1s then 3s sleeps).
pub(crate) fn plan_agent_retry(last_error: &str, attempt: u32) -> Result<AgentRetryOutcome, AgentError> {
    if agent_string_is_upgrade_plan(last_error) || agent_string_is_cannot_use_model(last_error) {
        return Err(AgentError(last_error.to_string()));
    }
    if !agent_string_is_retriable(last_error) || attempt >= MAX_AGENT_ATTEMPTS {
        return Ok(AgentRetryOutcome::StopRetrying);
    }
    let secs = if attempt == 1 { 1_u64 } else { 3_u64 };
    Ok(AgentRetryOutcome::Sleep(std::time::Duration::from_secs(secs)))
}

#[cfg(test)]
mod retry_policy_tests {
    use super::{
        agent_string_is_cannot_use_model, agent_string_is_retriable, agent_string_is_upgrade_plan,
        delimited_token_match, has_delimited_substring, is_identifier_byte, plan_agent_retry,
        retries_noun, timeout_word_without_identifier_false_positive, AgentRetryOutcome,
        MAX_AGENT_ATTEMPTS,
    };
    use std::time::Duration;

    #[test]
    fn upgrade_plan_substring_is_detected_case_insensitively() {
        assert!(agent_string_is_upgrade_plan(
            "Error: Upgrade Your Plan To Continue"
        ));
        assert!(!agent_string_is_upgrade_plan("timed out"));
    }

    #[test]
    fn upgrade_plan_errors_do_not_retry() {
        let msg = "billing: upgrade your plan to continue";
        let err = plan_agent_retry(msg, 1).expect_err("upgrade plan must fail fast");
        assert_eq!(err.0, msg);
    }

    #[test]
    fn cannot_use_model_errors_do_not_retry() {
        let msg = "Error: Cannot use this model with that provider";
        assert!(agent_string_is_cannot_use_model(msg));
        let err = plan_agent_retry(msg, 1).expect_err("invalid model must fail fast");
        assert_eq!(err.0, msg);
    }

    #[test]
    fn cannot_use_model_fails_fast_even_when_error_also_looks_retriable() {
        let msg = "rpc [unavailable]: Cannot use this model";
        let err = plan_agent_retry(msg, 1).expect_err("model error must beat retriable match");
        assert_eq!(err.0, msg);
    }

    #[test]
    fn retriable_timeout_delimited_without_timed_out_substring() {
        assert!(agent_string_is_retriable("rpc: connection timeout"));
        assert!(agent_string_is_retriable("session initialization failed"));
        assert!(!agent_string_is_retriable("timeoutable"));
        assert!(agent_string_is_retriable("rpc timeout"));
        assert!(agent_string_is_retriable("error: timeout"));
        assert!(!agent_string_is_retriable("atimeoutb"));
    }

    #[test]
    fn retriable_transient_errors_match_known_agent_strings() {
        assert!(agent_string_is_retriable("request timed out"));
        assert!(agent_string_is_retriable("DEADLINE EXCEEDED"));
        assert!(agent_string_is_retriable("WritableIterable is closed"));
        assert!(agent_string_is_retriable("child process is zombie"));
        assert!(agent_string_is_retriable("session/new failed"));
        assert!(agent_string_is_retriable("rpc [unavailable]"));
    }

    #[test]
    fn non_retriable_errors_stop_without_sleep() {
        assert!(!agent_string_is_retriable("invalid json"));
        let out = plan_agent_retry("invalid json", 1).unwrap();
        assert!(matches!(out, AgentRetryOutcome::StopRetrying), "{out:?}");
    }

    fn assert_retriable_sleep_secs(attempt: u32, expected_secs: u64) {
        let out = plan_agent_retry("timed out", attempt).unwrap();
        match out {
            AgentRetryOutcome::Sleep(d) => assert_eq!(d, Duration::from_secs(expected_secs)),
            AgentRetryOutcome::StopRetrying => {
                panic!("expected Sleep({expected_secs}s), got StopRetrying")
            }
        }
    }

    #[test]
    fn retriable_first_attempt_sleeps_one_second() {
        assert_retriable_sleep_secs(1, 1);
    }

    #[test]
    fn retriable_second_attempt_sleeps_three_seconds() {
        assert_retriable_sleep_secs(2, 3);
    }

    #[test]
    fn retriable_exhausts_after_max_agent_attempts() {
        let out = plan_agent_retry("timed out", MAX_AGENT_ATTEMPTS).unwrap();
        assert!(matches!(out, AgentRetryOutcome::StopRetrying), "{out:?}");
    }

    #[test]
    fn retries_noun_singular_and_plural() {
        assert_eq!(retries_noun(1), "retry");
        assert_eq!(retries_noun(2), "retries");
    }

    #[test]
    fn delimited_token_helpers_match_agent_timeout_edge_cases() {
        assert!(timeout_word_without_identifier_false_positive("rpc: connection timeout"));
        assert!(!timeout_word_without_identifier_false_positive("atimeoutb"));
        assert!(is_identifier_byte(b'a'));
        assert!(!is_identifier_byte(b' '));
        assert!(has_delimited_substring("session/new failed", "session/new"));
        assert!(delimited_token_match("init session", "session"));
    }
}
