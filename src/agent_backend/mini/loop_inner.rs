//! Inner bash-fence loop for one `run_coder_prompt`.

use super::loop_inner_phases::{
    run_investigate_turn, run_wind_down_turn, InvestigateStep, WindDownStep,
};
use super::loop_inner_prompt::push_user_prompt;
use super::loop_inner_types::{CompleteTurnRequest, LoopCounters, LoopPhase};
use super::loop_types::{LoopDriverOutcome, LoopDriverRun};
use crate::agent_backend::mini::retry_fork::MiniRetryStrategy;
use crate::acp::AgentError;

pub async fn run_inner_loop(run: LoopDriverRun<'_>) -> Result<LoopDriverOutcome, AgentError> {
    let LoopDriverRun {
        llm,
        session,
        user_prompt,
        config,
        trace,
        timing,
        llm_phase,
        single_attempt,
        gate_attempt,
        retry_strategy,
    } = run;
    if should_push_user_prompt(gate_attempt, retry_strategy) {
        push_user_prompt(session, config, user_prompt);
    }
    session.bash_commands_this_prompt.clear();

    let mut counters = LoopCounters {
        http_turn_count: 0,
        bash_exec_count: 0,
        investigate_http_turns: 0,
        had_bash_this_prompt: false,
    };
    let mut phase = LoopPhase::Investigate;
    let mut transcript = String::new();

    loop {
        let mut turn_req = CompleteTurnRequest {
            llm,
            session,
            config,
            trace,
            timing,
            llm_phase,
            single_attempt,
        };
        match phase {
            LoopPhase::Investigate => {
                phase = match run_investigate_phase(&mut turn_req, &mut counters, &mut transcript)
                    .await?
                {
                    InvestigatePhaseResult::Continue(phase) => phase,
                    InvestigatePhaseResult::Done(outcome) => return Ok(outcome),
                    InvestigatePhaseResult::Failed(err) => return Err(err),
                };
            }
            LoopPhase::WindDown => {
                match run_wind_down_phase(&mut turn_req, &mut counters, &mut transcript).await? {
                    WindDownPhaseResult::Done(outcome) => return Ok(outcome),
                    WindDownPhaseResult::Failed(err) => return Err(err),
                }
            }
        }
    }
}

enum InvestigatePhaseResult {
    Continue(LoopPhase),
    Done(LoopDriverOutcome),
    Failed(AgentError),
}

enum WindDownPhaseResult {
    Done(LoopDriverOutcome),
    Failed(AgentError),
}

fn should_push_user_prompt(gate_attempt: u32, retry_strategy: MiniRetryStrategy) -> bool {
    gate_attempt <= 1 || retry_strategy != MiniRetryStrategy::CumulativeTranscript
}

async fn run_investigate_phase(
    turn_req: &mut CompleteTurnRequest<'_>,
    counters: &mut LoopCounters,
    transcript: &mut String,
) -> Result<InvestigatePhaseResult, AgentError> {
    use crate::agent_backend::mini::terminal::{MiniPhase, MiniTerminalReason};
    use super::loop_inner_finish::{finish_exhausted, TerminalEmitCtx};

    let config = turn_req.config;
    let trace = turn_req.trace;

    if counters.investigate_http_turns >= config.max_http_turns {
        if !counters.had_bash_this_prompt {
            return Ok(InvestigatePhaseResult::Failed(finish_exhausted(
                trace,
                config.max_http_turns,
                transcript,
                TerminalEmitCtx {
                    reason: MiniTerminalReason::BudgetExhaustedBeforeClassification,
                    http_turn_count: counters.http_turn_count,
                    bash_exec_count: counters.bash_exec_count,
                    phase_at_exit: MiniPhase::Investigate,
                },
            )));
        }
        return Ok(InvestigatePhaseResult::Continue(LoopPhase::WindDown));
    }
    match run_investigate_turn(turn_req, counters, transcript).await? {
        InvestigateStep::Continue => Ok(InvestigatePhaseResult::Continue(LoopPhase::Investigate)),
        InvestigateStep::SwitchToWindDown => {
            Ok(InvestigatePhaseResult::Continue(LoopPhase::WindDown))
        }
        InvestigateStep::Finished(outcome) => Ok(InvestigatePhaseResult::Done(outcome)),
        InvestigateStep::Failed(err) => Ok(InvestigatePhaseResult::Failed(err)),
    }
}

async fn run_wind_down_phase(
    turn_req: &mut CompleteTurnRequest<'_>,
    counters: &mut LoopCounters,
    transcript: &mut String,
) -> Result<WindDownPhaseResult, AgentError> {
    match run_wind_down_turn(turn_req, counters, transcript).await? {
        WindDownStep::Finished(outcome) => Ok(WindDownPhaseResult::Done(outcome)),
        WindDownStep::Failed(err) => Ok(WindDownPhaseResult::Failed(err)),
    }
}
