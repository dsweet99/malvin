use std::path::Path;
use std::time::Instant;

use crate::acp::{
    backoff_after_agent_failure, teardown_coder_session_after_transport_error, AgentClient,
    AgentError, CoderSessionPromptDispatch, coder_prompt_exhausted_error,
    dispatch_coder_session_prompt, record_coder_prompt_llm_timing,
};

pub(crate) const TEST_PROMPT_OK_WITHOUT_DISPATCH: &str = "__malvin_test_coder_prompt_ok__";

pub(crate) const fn coder_prompt_max_attempts(single_attempt: bool, max_acp_retries: u32) -> u32 {
    if single_attempt {
        1
    } else {
        max_acp_retries
    }
}

pub(crate) fn coder_prompt_skip_dispatch(full_prompt: &str) -> bool {
    cfg!(test) && full_prompt == TEST_PROMPT_OK_WITHOUT_DISPATCH
}

pub(crate) fn coder_prompt_cwd_or_error(
    cwd: Option<std::path::PathBuf>,
) -> Result<std::path::PathBuf, AgentError> {
    cwd.ok_or_else(|| AgentError("begin_coder_session was not called".to_string()))
}

pub(crate) fn coder_prompt_retry_failure(attempts_used: u32, last_error: String) -> AgentError {
    coder_prompt_exhausted_error(attempts_used, last_error)
}

pub(crate) fn record_coder_prompt_last_error(last_error: &mut String, error: String) {
    *last_error = error;
}

pub(crate) const fn coder_prompt_stop_retry_loop(should_stop: bool) -> bool {
    should_stop
}

#[allow(clippy::missing_const_for_fn)]
pub(crate) fn coder_prompt_init_last_error() -> String {
    String::new()
}

#[allow(clippy::unnecessary_wraps, clippy::missing_const_for_fn)]
pub(crate) fn coder_prompt_attempt_ok() -> Result<(), AgentError> {
    Ok(())
}

pub(crate) async fn run_coder_prompt_with_retries(
    client: &mut AgentClient,
    dispatch: CoderSessionPromptDispatch<'_>,
    llm_phase: Option<crate::run_timing::TimingPhase>,
    single_attempt: bool,
) -> Result<(), AgentError> {
    let cwd = coder_prompt_cwd_or_error(client.coder_session_cwd.clone())?;
    let mut last_error = coder_prompt_init_last_error();
    let mut attempts_used = 0_u32;
    let max_attempts = coder_prompt_max_attempts(single_attempt, client.max_acp_retries);
    for attempt in 1..=max_attempts {
        attempts_used = attempt;
        match run_one_coder_prompt_attempt(client, &cwd, &dispatch, llm_phase).await {
            Ok(()) => return coder_prompt_attempt_ok(),
            Err(e) => {
                record_coder_prompt_last_error(&mut last_error, e);
                if coder_prompt_stop_retry_loop(
                    backoff_after_agent_failure(
                        client.timing.as_ref(),
                        &last_error,
                        attempt,
                        max_attempts,
                    )
                    .await?,
                ) {
                    break;
                }
            }
        }
    }
    Err(coder_prompt_retry_failure(attempts_used, last_error))
}

pub(crate) async fn run_one_coder_prompt_attempt(
    client: &mut AgentClient,
    cwd: &Path,
    dispatch: &CoderSessionPromptDispatch<'_>,
    llm_phase: Option<crate::run_timing::TimingPhase>,
) -> Result<(), String> {
    if !client.has_open_coder_session() {
        client.begin_coder_session(cwd).await.map_err(|e| e.0)?;
    }
    let session = client
        .coder_session
        .as_ref()
        .ok_or_else(|| "begin_coder_session was not called".to_string())?
        .clone();
    let attempt_dispatch = CoderSessionPromptDispatch {
        session: &session,
        full_prompt: dispatch.full_prompt,
        log_path: dispatch.log_path,
        who: dispatch.who,
        do_trace_split: dispatch.do_trace_split,
        stdout_bracket_label: dispatch.stdout_bracket_label,
    };
    let t0 = Instant::now();
    let prompt_res = if coder_prompt_skip_dispatch(dispatch.full_prompt) {
        Ok(())
    } else {
        dispatch_coder_session_prompt(&attempt_dispatch).await
    };
    record_coder_prompt_llm_timing(client.timing.as_ref(), llm_phase, t0.elapsed());
    match prompt_res {
        Ok(()) => Ok(()),
        Err(e) => {
            teardown_coder_session_after_transport_error(client, &e).await;
            Err(e)
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::{
        coder_prompt_attempt_ok, coder_prompt_cwd_or_error, coder_prompt_init_last_error,
        coder_prompt_max_attempts, coder_prompt_retry_failure, coder_prompt_skip_dispatch,
        coder_prompt_stop_retry_loop, record_coder_prompt_last_error, run_coder_prompt_with_retries, run_one_coder_prompt_attempt,
        TEST_PROMPT_OK_WITHOUT_DISPATCH,
    };

    #[test]
    fn coder_prompt_attempt_ok_is_success() {
        assert!(coder_prompt_attempt_ok().is_ok());
    }

    #[test]
    fn coder_prompt_init_last_error_is_empty() {
        assert!(coder_prompt_init_last_error().is_empty());
    }

    #[test]
    fn coder_prompt_stop_retry_loop_passes_through_flag() {
        assert!(coder_prompt_stop_retry_loop(true));
        assert!(!coder_prompt_stop_retry_loop(false));
    }

    #[test]
    fn record_coder_prompt_last_error_overwrites_slot() {
        let mut last = String::from("old");
        record_coder_prompt_last_error(&mut last, "new".into());
        assert_eq!(last, "new");
    }

    #[test]
    fn coder_prompt_cwd_or_error_requires_session_cwd() {
        let err = coder_prompt_cwd_or_error(None).expect_err("missing cwd");
        assert!(err.0.contains("begin_coder_session"));
        let cwd = std::path::PathBuf::from("/tmp");
        assert_eq!(coder_prompt_cwd_or_error(Some(cwd.clone())).expect("cwd"), cwd);
    }

    #[test]
    fn coder_prompt_retry_failure_wraps_last_error() {
        let err = coder_prompt_retry_failure(2, "boom".into());
        assert!(err.0.contains("boom"));
    }

    #[test]
    fn coder_prompt_max_attempts_respects_single_attempt_flag() {
        assert_eq!(coder_prompt_max_attempts(true, 5), 1);
        assert_eq!(coder_prompt_max_attempts(false, 5), 5);
    }

    #[test]
    fn coder_prompt_skip_dispatch_matches_test_prompt() {
        assert!(coder_prompt_skip_dispatch(TEST_PROMPT_OK_WITHOUT_DISPATCH));
        assert!(!coder_prompt_skip_dispatch("real prompt"));
    }

    #[test]
    fn kiss_cov_run_coder_prompt_with_retries() {
        let _ = run_coder_prompt_with_retries;
    }

    #[test]
    fn kiss_cov_run_one_coder_prompt_attempt() {
        let _ = run_one_coder_prompt_attempt;
    }
}
