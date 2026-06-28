//! Gate-iteration retry loop for [`super::client::MiniAgentClient`].

use super::client::MiniAgentClient;
use super::client_gate_retry_attempt::run_one_gate_attempt;
use super::loop_driver::LoopDriverConfig;
use super::terminal::{MiniPhase, MiniTerminalReason, MiniTerminalRecord};
use super::trace_audit::{emit_retry_fork, emit_terminal};
use crate::acp::{
    backoff_after_agent_failure, retries_noun, AgentError, CoderPromptOptions,
};

pub(crate) struct ForkLedgerBuild {
    pub(super) prompt_index: u32,
    pub(super) attempt: u32,
    pub(super) message_checkpoint_len: usize,
    pub(super) workspace_manifest_hash: String,
    pub(super) bash_commands: Vec<String>,
    pub(super) outcome: super::retry_fork::ForkOutcome,
    pub(super) strategy: super::retry_fork::MiniRetryStrategy,
}

pub(crate) struct GateAttemptOutcome {
    pub(super) success_text: Option<String>,
    pub(super) failure_reason: String,
    pub(super) ledger: super::retry_fork::RetryForkLedger,
}

pub(crate) struct GateAttemptRun<'a> {
    pub(super) prompt: &'a str,
    pub(super) driver_config: &'a LoopDriverConfig,
    pub(super) llm_phase: Option<crate::run_timing::TimingPhase>,
    pub(super) single_attempt: bool,
    pub(super) attempt: u32,
}

pub(crate) struct GateRetryStopCheck<'a> {
    single_attempt: bool,
    timing: Option<&'a std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
    last_error: &'a str,
    attempt: u32,
    max_attempts: u32,
}

pub(crate) async fn run_coder_prompt_with_gate_retries(
    client: &mut MiniAgentClient,
    prompt: &str,
    driver_config: &LoopDriverConfig,
    opts: CoderPromptOptions<'_>,
) -> Result<(), AgentError> {
    let max_attempts = if opts.single_attempt {
        1
    } else {
        client.config.max_gate_retries.max(1)
    };
    let mut last_error = String::new();
    let mut attempts_used = 0_u32;
    for attempt in 1..=max_attempts {
        attempts_used = attempt;
        let attempt_outcome = run_one_gate_attempt(
            client,
            GateAttemptRun {
                prompt,
                driver_config,
                llm_phase: opts.llm_phase,
                single_attempt: opts.single_attempt,
                attempt,
            },
        )
        .await?;
        emit_retry_fork(&client.trace, &attempt_outcome.ledger);
        if let Some(text) = attempt_outcome.success_text {
            client.last_response = Some(text);
            return Ok(());
        }
        last_error = attempt_outcome.failure_reason;
        if should_stop_gate_retries(GateRetryStopCheck {
            single_attempt: opts.single_attempt,
            timing: client.timing.as_ref(),
            last_error: &last_error,
            attempt,
            max_attempts,
        })
        .await?
        {
            return fail_gate_exhausted_with_error(client, &last_error);
        }
    }
    let retries = attempts_used.saturating_sub(1);
    let noun = retries_noun(retries);
    Err(AgentError(format!(
        "mini agent (gate_iteration) failed after {retries} {noun}. Last error:\n{last_error}"
    )))
}

pub(crate) async fn should_stop_gate_retries(
    check: GateRetryStopCheck<'_>,
) -> Result<bool, AgentError> {
    if check.single_attempt {
        return Ok(true);
    }
    backoff_after_agent_failure(
        check.timing,
        check.last_error,
        check.attempt,
        check.max_attempts,
    )
    .await
}

fn fail_gate_exhausted_with_error(
    client: &MiniAgentClient,
    last_error: &str,
) -> Result<(), AgentError> {
    let record = MiniTerminalRecord::new(
        MiniTerminalReason::GateIterationExhausted,
        0,
        0,
        MiniPhase::Terminal,
    );
    emit_terminal(&client.trace, &record);
    Err(AgentError(last_error.to_string()))
}

#[cfg(test)]
mod gate_retry_stop_tests {
    use super::*;
    use crate::agent_backend::mini::LoopDriverConfig;

    #[test]
    fn kiss_witness_gate_attempt_run_and_stop_check() {
        let config = LoopDriverConfig {
            max_http_turns: 4,
            max_bash_execs: 128,
            max_http_retries: 1,
            max_shrink_passes: 0,
            mini_constraints: "c",
            expects_investigation: false,
        };
        let run = GateAttemptRun {
            prompt: "p",
            driver_config: &config,
            llm_phase: None,
            single_attempt: false,
            attempt: 1,
        };
        let GateAttemptRun {
            prompt,
            attempt,
            ..
        } = run;
        assert_eq!(prompt, "p");
        assert_eq!(attempt, 1);
        let stop = GateRetryStopCheck {
            single_attempt: true,
            timing: None,
            last_error: "e",
            attempt: 1,
            max_attempts: 2,
        };
        let GateRetryStopCheck {
            last_error,
            max_attempts,
            ..
        } = stop;
        assert_eq!(last_error, "e");
        assert_eq!(max_attempts, 2);
    }

    #[tokio::test]
    async fn gate_retry_stop_single_attempt_returns_true() {
        let stop = should_stop_gate_retries(GateRetryStopCheck {
            single_attempt: true,
            timing: None,
            last_error: "fail",
            attempt: 1,
            max_attempts: 3,
        })
        .await
        .expect("stop check");
        assert!(stop);
    }

    #[tokio::test]
    async fn gate_retry_stop_multi_attempt_continues_before_max() {
        let stop = should_stop_gate_retries(GateRetryStopCheck {
            single_attempt: false,
            timing: None,
            last_error: "fail",
            attempt: 1,
            max_attempts: 2,
        })
        .await
        .expect("stop check");
        assert!(!stop);
    }

    #[tokio::test]
    async fn gate_retry_stop_at_max_attempts_returns_true() {
        let stop = should_stop_gate_retries(GateRetryStopCheck {
            single_attempt: false,
            timing: None,
            last_error: "exhausted",
            attempt: 2,
            max_attempts: 2,
        })
        .await
        .expect("stop check");
        assert!(stop);
    }
}
