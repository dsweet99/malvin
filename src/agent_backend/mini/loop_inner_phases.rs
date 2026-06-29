//! Investigate and `WindDown` phase turns for the inner bash-fence loop.

use crate::agent_backend::mini::terminal::{MiniPhase, MiniTerminalReason};
use super::loop_inner_bash::{append_bash_observation, BashObservationInput};
use super::loop_inner_classify::classify_turn;
use super::loop_inner_finish::{
    append_assistant_message, finish_done_turn, finish_exhausted, TerminalEmitCtx,
};
use super::loop_inner_http::complete_turn_with_recovery;
use super::loop_inner_types::{CompleteTurnRequest, LoopCounters, TurnAction};
use super::loop_types::LoopDriverOutcome;
use crate::acp::AgentError;
use crate::nested_budget_scopes::BudgetScopeLayer;

pub(crate) enum InvestigateStep {
    Continue,
    SwitchToWindDown,
    Finished(LoopDriverOutcome),
    Failed(AgentError),
}

pub(crate) enum WindDownStep {
    Finished(LoopDriverOutcome),
    Failed(AgentError),
}

pub(crate) async fn run_investigate_turn(
    req: &mut CompleteTurnRequest<'_>,
    counters: &mut LoopCounters,
    transcript: &mut String,
) -> Result<InvestigateStep, AgentError> {
    let response = complete_turn_with_recovery(req, counters).await?;
    counters.http_turn_count += 1;
    counters.investigate_http_turns += 1;

    let assistant_text = response.content.clone();
    append_assistant_message(req.session, transcript, &assistant_text);

    let (action, warnings) =
        classify_turn(&assistant_text, req.config, counters.had_bash_this_prompt);
    match action {
        TurnAction::Done(reason) => {
            let outcome = finish_done_turn(
                req.trace,
                &assistant_text,
                response.reasoning.as_deref(),
                TerminalEmitCtx {
                    reason,
                    http_turn_count: counters.http_turn_count,
                    bash_exec_count: counters.bash_exec_count,
                    phase_at_exit: MiniPhase::Terminal,
                },
            );
            Ok(InvestigateStep::Finished(outcome))
        }
        TurnAction::RunBash(fences) => {
            let input = BashTurnInput {
                assistant_text: &assistant_text,
                reasoning: response.reasoning.as_deref(),
                fences: &fences,
                stream_warnings: !warnings.is_empty(),
            };
            investigate_bash_turn(req, counters, transcript, input)
        }
    }
}

pub(crate) struct BashTurnInput<'a> {
    assistant_text: &'a str,
    reasoning: Option<&'a str>,
    fences: &'a [crate::agent_backend::mini::fence_parser::BashFence],
    stream_warnings: bool,
}

fn investigate_bash_turn(
    req: &mut CompleteTurnRequest<'_>,
    counters: &mut LoopCounters,
    transcript: &mut String,
    input: BashTurnInput<'_>,
) -> Result<InvestigateStep, AgentError> {
    if input.stream_warnings {
        if req.trace.plain_lines {
            req.trace.record_assistant_audit(input.assistant_text);
        } else {
            req.trace.stream_assistant_chunks(input.assistant_text);
        }
    }
    counters.had_bash_this_prompt = true;
    let fence_count = u32::try_from(input.fences.len()).unwrap_or(u32::MAX);
    if counters.bash_exec_count + fence_count
        > BudgetScopeLayer::MiniBashExec.effective_max_attempts(req.config.max_bash_execs, req.single_attempt)
    {
        let err = finish_exhausted(
            req.trace,
            req.config.max_http_turns,
            transcript,
            TerminalEmitCtx {
                reason: MiniTerminalReason::BudgetExhaustedBashExecs,
                http_turn_count: counters.http_turn_count,
                bash_exec_count: counters.bash_exec_count,
                phase_at_exit: MiniPhase::Investigate,
            },
        );
        return Ok(InvestigateStep::Failed(err));
    }
    if req.trace.plain_lines {
        req.trace.record_assistant_audit(input.assistant_text);
    } else {
        req.trace.stream_assistant_chunks(input.assistant_text);
    }
    if let Some(r) = input.reasoning {
        req.trace.mini_thought(r);
    }
    append_bash_observation(
        input.fences,
        BashObservationInput {
            session: req.session,
            trace: req.trace,
            transcript,
            counters,
        },
    )?;
    if counters.investigate_http_turns
        >= BudgetScopeLayer::MiniHttpTurn.effective_max_attempts(req.config.max_http_turns, req.single_attempt)
    {
        Ok(InvestigateStep::SwitchToWindDown)
    } else {
        Ok(InvestigateStep::Continue)
    }
}

pub(crate) async fn run_wind_down_turn(
    req: &mut CompleteTurnRequest<'_>,
    counters: &mut LoopCounters,
    transcript: &mut String,
) -> Result<WindDownStep, AgentError> {
    let response = complete_turn_with_recovery(req, counters).await?;
    counters.http_turn_count += 1;

    let assistant_text = response.content.clone();
    append_assistant_message(req.session, transcript, &assistant_text);

    let (action, _) = classify_turn(&assistant_text, req.config, counters.had_bash_this_prompt);
    match action {
        TurnAction::Done(reason) => {
            let reason = if reason == MiniTerminalReason::FencelessPremature {
                MiniTerminalReason::FencelessComplete
            } else {
                reason
            };
            let outcome = finish_done_turn(
                req.trace,
                &assistant_text,
                response.reasoning.as_deref(),
                TerminalEmitCtx {
                    reason,
                    http_turn_count: counters.http_turn_count,
                    bash_exec_count: counters.bash_exec_count,
                    phase_at_exit: MiniPhase::WindDown,
                },
            );
            Ok(WindDownStep::Finished(outcome))
        }
        TurnAction::RunBash(_) => {
            let err = finish_exhausted(
                req.trace,
                req.config.max_http_turns,
                transcript,
                TerminalEmitCtx {
                    reason: MiniTerminalReason::BudgetExhaustedAfterBashOnLastHttpTurn,
                    http_turn_count: counters.http_turn_count,
                    bash_exec_count: counters.bash_exec_count,
                    phase_at_exit: MiniPhase::WindDown,
                },
            );
            Ok(WindDownStep::Failed(err))
        }
    }
}
