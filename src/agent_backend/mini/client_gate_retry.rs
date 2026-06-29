//! Gate-iteration retry loop for [`super::client::MiniAgentClient`].

use super::client::MiniAgentClient;
use super::client_gate_retry_attempt::run_one_gate_attempt;
use super::loop_driver::LoopDriverConfig;
use super::terminal::{MiniPhase, MiniTerminalReason, MiniTerminalRecord};
use super::trace_audit::{emit_retry_fork, emit_terminal};
use crate::acp::{
    backoff_after_mini_gate_failure, retries_noun, AgentError, CoderPromptOptions,
};
use crate::fork_state::ForkState;
use crate::nested_budget_scopes::BudgetScopeLayer;

pub(crate) struct ForkLedgerBuild {
    pub(super) prompt_index: u32,
    pub(super) attempt: u32,
    pub(super) checkpoint: ForkState,
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
    let layer = BudgetScopeLayer::MiniGateIteration;
    let max_attempts = layer.effective_max_attempts(client.config.max_gate_retries, opts.single_attempt);
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
    backoff_after_mini_gate_failure(
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
#[path = "client_gate_retry_test.rs"]
mod gate_retry_stop_tests;
