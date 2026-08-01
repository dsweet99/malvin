//! Single gate-attempt execution for Mini gate retries.

use crate::mini_agent::MiniAgentClient;
use super::mini_gate_retry::{
    ForkLedgerBuild, GateAttemptOutcome, GateAttemptRun,
};
use crate::mini_agent::{run_inner_loop, LoopDriverRun, LoopDriverSession};
use crate::mini_agent::retry_fork::{
    build_divergence_observation, ForkOutcome, MiniRetryStrategy, RetryForkLedger,
};
use crate::acp::AgentError;
use crate::fork_state::ForkState;

pub(crate) async fn run_one_gate_attempt(
    client: &mut MiniAgentClient,
    run: GateAttemptRun<'_>,
) -> Result<GateAttemptOutcome, AgentError> {
    let GateAttemptRun {
        prompt,
        driver_config,
        llm_phase,
        single_attempt,
        attempt,
    } = run;
    let session = client.session.as_mut().expect("session checked above");
    let checkpoint = ForkState::capture(
        session.cwd.as_path(),
        &session.history,
        &session.previous_response,
    );
    session.bash_commands_this_prompt.clear();

    let result = run_inner_loop(LoopDriverRun {
        llm: &client.llm,
        session,
        user_prompt: prompt,
        config: driver_config,
        trace: &client.trace,
        timing: client.timing.as_ref(),
        llm_phase,
        single_attempt,
        gate_attempt: attempt,
        retry_strategy: client.config.retry_strategy,
    })
    .await;

    let session = client.session.as_mut().expect("session checked above");
    let bash_commands = session.bash_commands_this_prompt.clone();
    let (outcome_ok, failure_reason) = match &result {
        Ok(_) => (true, String::new()),
        Err(e) => (false, e.0.clone()),
    };
    let ledger = build_fork_ledger(ForkLedgerBuild {
        prompt_index: session.prompt_index,
        attempt,
        checkpoint,
        bash_commands,
        outcome: if outcome_ok {
            ForkOutcome::Succeeded
        } else {
            ForkOutcome::Failed
        },
        strategy: client.config.retry_strategy,
    });

    if let Ok(outcome) = result {
        return Ok(GateAttemptOutcome {
            success_text: Some(outcome.final_assistant_text),
            failure_reason: String::new(),
            ledger,
        });
    }

    apply_retry_strategy(client.config.retry_strategy, session, &ledger, &failure_reason);
    Ok(GateAttemptOutcome {
        success_text: None,
        failure_reason,
        ledger,
    })
}

pub(super) fn build_fork_ledger(input: ForkLedgerBuild) -> RetryForkLedger {
    RetryForkLedger {
        prompt_index: input.prompt_index,
        attempt: input.attempt,
        history: input.checkpoint.history.clone(),
        previous_response: input.checkpoint.previous_response.clone(),
        workspace_manifest_hash: input.checkpoint.workspace_manifest_hash.clone(),
        bash_commands: input.bash_commands,
        outcome: input.outcome,
        strategy: input.strategy,
    }
}

fn apply_retry_strategy(
    retry_strategy: MiniRetryStrategy,
    session: &mut LoopDriverSession,
    ledger: &RetryForkLedger,
    last_error: &str,
) {
    let checkpoint = ledger.checkpoint();
    match retry_strategy {
        MiniRetryStrategy::CumulativeTranscript => {
            let obs = build_divergence_observation(
                &session.bash_commands_this_prompt,
                last_error,
                &checkpoint.workspace_manifest_hash,
            );
            session.pending_new_request = Some(obs);
            session.section_shape_nudged = false;
        }
        MiniRetryStrategy::WorkspaceSnapshot => {
            session.history = checkpoint.history;
            session.previous_response = checkpoint.previous_response;
            session.pending_new_request = None;
            session.section_shape_nudged = false;
        }
    }
}
