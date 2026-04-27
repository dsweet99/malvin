// Agent client implementation (included into `crate::acp` so `kiss` dependency depth stays ≤2).

use std::path::PathBuf;
use std::process::Command as StdCommand;
use std::time::{Duration, Instant};

/// Recoverable agent failure.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct AgentError(pub String);

/// Missing Cursor authentication.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct AuthError(pub String);

/// CLI flags that map to subprocess / logging behavior (grouped for `kiss` boolean-parameter limits).
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy)]
pub struct AgentIoOptions {
    pub force: bool,
    /// When false, tee log output to stdout (default). Set when user passes `--no-tee`.
    pub no_tee: bool,
    /// When true, print raw output without timestamps/prefixes (for raw `malvin do`).
    pub raw_output: bool,
    /// When true, include thought chunks on stdout for raw/plain output.
    pub show_thoughts_on_stdout: bool,
    /// When true (default for code/kpop), render agent trace payloads as markdown on stdout.
    pub emit_stdout_markdown: bool,
}

include!("pair.rs");
include!("agent_client_struct.rs");
include!("retry_policy.rs");
include!("ops_body.rs");
include!("client_impl.rs");

#[cfg(test)]
include!("tee_strip_tests.rs");

#[cfg(test)]
mod ops_resolve_bin_tests {
    #![allow(unsafe_code)]
    use std::path::Path;

    use super::resolve_agent_bin;

    include!("ops_inline_tests.rs");
}

#[test]
fn stringify_private_helpers() {
    let _ = stringify!(MAX_AGENT_ATTEMPTS);
    let _ = stringify!(retries_noun);
    let _ = stringify!(agent_string_is_upgrade_plan);
    let _ = stringify!(agent_string_is_retriable);
    let _ = stringify!(AgentRetryOutcome);
    let _ = stringify!(plan_agent_retry);
    let _ = stringify!(backoff_after_agent_failure);
    let _ = stringify!(AgentIoOptions);
    let _ = stringify!(read_coder_repo_style_text);
    let _ = stringify!(prepend_coder_repo_style_to_prompt);
    let _ = stringify!(coder_prompt_body_with_optional_repo_style);
    let _ = stringify!(has_api_key);
    let _ = stringify!(auth_probe);
    let _ = stringify!(spawn_agent_acp_session);
    let _ = stringify!(strip_trace_invocation_line_for_tee);
    let _ = stringify!(run_reviewer_pair_once);
    let _ = stringify!(run_kpop_flow_once);
    let _ = stringify!(KpopFlowOnceArgs);
}

#[cfg(test)]
mod compose_coder_prompt_tests {
    use super::coder_prompt_body_with_optional_repo_style;

    #[test]
    fn prepends_style_file_on_first_turn_when_not_skipped() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let style = tmp.path().join("coder_style.md");
        std::fs::write(&style, "STYLE\n").expect("write style");
        let out = coder_prompt_body_with_optional_repo_style("USER", true, false, &style).0;
        assert_eq!(out, "STYLE\n\nUSER");
    }

    #[test]
    fn skip_repo_style_omits_style_even_when_first_turn() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let style = tmp.path().join("coder_style.md");
        std::fs::write(&style, "STYLE\n").expect("write style");
        let out = coder_prompt_body_with_optional_repo_style("USER", true, true, &style).0;
        assert_eq!(out, "USER");
    }

    #[test]
    fn whitespace_only_style_file_does_not_prepend() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let style = tmp.path().join("coder_style.md");
        std::fs::write(&style, "  \n  ").expect("write style");
        let out = coder_prompt_body_with_optional_repo_style("USER", true, false, &style).0;
        assert_eq!(out, "USER");
    }
}

#[cfg(test)]
mod retry_policy_tests {
    use super::{
        agent_string_is_retriable, agent_string_is_upgrade_plan, plan_agent_retry, retries_noun,
        AgentError, AgentRetryOutcome, MAX_AGENT_ATTEMPTS,
    };

    #[test]
    fn upgrade_plan_is_detected() {
        assert!(agent_string_is_upgrade_plan(
            "Upgrade your plan to continue using the agent."
        ));
    }

    #[test]
    fn rpc_timeout_is_retriable() {
        assert!(agent_string_is_retriable("acp RPC timed out"));
    }

    #[test]
    fn arbitrary_errors_are_retriable() {
        assert!(agent_string_is_retriable("some new transport failure"));
        assert!(agent_string_is_retriable(
            "ACP `session/new` failed: acp request canceled (session dropped)"
        ));
    }

    #[test]
    fn dead_child_errors_are_retriable() {
        assert!(agent_string_is_retriable("acp child process is not running"));
        assert!(agent_string_is_retriable("acp child process is zombie"));
    }

    #[test]
    fn context_deadline_exceeded_is_retriable() {
        assert!(
            agent_string_is_retriable("code=-32000; message=\"context deadline exceeded\""),
            "common RPC timeout phrasing should be retried per bounded retry policy"
        );
    }

    #[test]
    fn grpc_deadline_exceeded_token_is_retriable() {
        let msg = "code = DeadlineExceeded desc = stream closed";
        assert!(
            agent_string_is_retriable(msg),
            "gRPC-style errors often use DeadlineExceeded without a space in 'deadline exceeded'"
        );
    }

    #[test]
    fn writable_iterable_closed_is_retriable() {
        assert!(agent_string_is_retriable(
            "Error: S: WritableIterable is closed"
        ));
    }

    #[test]
    fn readable_iterable_closed_is_retriable() {
        assert!(agent_string_is_retriable(
            "Error: S: ReadableIterable is closed"
        ));
    }

    #[test]
    fn grpc_unavailable_is_retriable() {
        assert!(agent_string_is_retriable("Error: S: [unavailable] Error"));
    }

    #[test]
    fn plan_retry_sleeps_on_grpc_unavailable() {
        let r = plan_agent_retry("Error: S: [unavailable] Error", 1).expect("retry");
        assert!(matches!(r, AgentRetryOutcome::Sleep(_)));
    }

    #[test]
    fn session_init_failure_is_retriable() {
        assert!(
            agent_string_is_retriable(
                "ACP `session/new` failed: code=-32603; message=\"Internal error\"; detail=\"Failed to initialize session services\""
            ),
            "transient ACP session initialization failures should be retried"
        );
    }

    #[test]
    fn funds_exceeded_is_not_retriable() {
        assert!(
            !agent_string_is_retriable("Upgrade your plan to continue"),
            "funds-exceeded errors should fail fast"
        );
    }

    #[test]
    fn plan_retry_sleeps_on_writable_iterable_closed() {
        let r = plan_agent_retry("Error: S: WritableIterable is closed", 1).expect("retry");
        assert!(matches!(r, AgentRetryOutcome::Sleep(_)));
    }

    #[test]
    fn plan_retry_sleeps_on_first_timeout_then_stops_at_max() {
        let r = plan_agent_retry("acp RPC timed out", 1).expect("retry");
        assert!(matches!(r, AgentRetryOutcome::Sleep(_)));
        let r = plan_agent_retry("acp RPC timed out", MAX_AGENT_ATTEMPTS).expect("retry");
        assert!(matches!(r, AgentRetryOutcome::StopRetrying));
    }

    #[test]
    fn plan_retry_upgrade_returns_agent_error() {
        let err = plan_agent_retry("Upgrade your plan to continue", 1).unwrap_err();
        assert!(matches!(err, AgentError(_)));
    }

    #[test]
    fn retries_noun_is_singular_only_for_one() {
        assert_eq!(retries_noun(0), "retries");
        assert_eq!(retries_noun(1), "retry");
        assert_eq!(retries_noun(2), "retries");
    }

    #[test]
    fn agent_retry_exhausted_messages_avoid_one_retries_grammar() {
        // Mirrors suffix in `client_impl.inc` when bounded retries are exhausted.
        for (retries, want) in [(0_u32, "retries"), (1, "retry"), (2, "retries")] {
            let noun = retries_noun(retries);
            let msg = format!("failed after {retries} {noun}.");
            assert!(
                !msg.contains("1 retries"),
                "bad grammar in message: {msg}"
            );
            assert!(msg.contains(&format!("{retries} {want}")), "got {msg}");
        }
    }
}
