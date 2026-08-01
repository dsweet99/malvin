//! HTTP completion and context-recovery for the inner bash-fence loop.

use std::time::Instant;

use crate::mini_agent::protocol::{parse_history_response, SectionParseError, SECTION_SHAPE_NUDGE};

use crate::mini_agent::context_recovery::shrink_session_memory;
use crate::mini_agent::memory_assemble::{
    assemble_session_messages, build_sticky_header, SessionAssemble,
};
use crate::mini_agent::terminal::{MiniPhase, MiniTerminalReason, MiniTerminalRecord};
use crate::mini_agent::trace_audit::{
    emit_prompt_shrink, emit_prompt_shrink_stalled, emit_terminal,
};
use super::loop_http::{complete_transport_with_retries, HttpCompletionError, HttpRetryRequest};
use super::loop_inner_types::{CompleteTurnRequest, LoopCounters};
use crate::acp::AgentError;
use crate::nested_budget_scopes::BudgetScopeLayer;
use crate::run_timing::{record_llm, TimingPhase};

pub(crate) struct ConsolidatedTurn {
    pub new_history: String,
    pub response: String,
    pub reasoning: Option<String>,
}

pub(crate) async fn complete_turn_with_recovery(
    req: &mut CompleteTurnRequest<'_>,
    counters: &LoopCounters,
) -> Result<ConsolidatedTurn, AgentError> {
    let mut shrink_passes_used = 0_u32;
    loop {
        match complete_and_parse_turn(req).await {
            Ok(r) => return Ok(r),
            Err(TurnFail::ContextOverflow) => {
                handle_overflow(req, counters, &mut shrink_passes_used)?;
            }
            Err(TurnFail::HttpExhausted(msg)) => {
                return Err(terminal_err(req, counters, MiniTerminalReason::HttpRetryExhausted, msg));
            }
            Err(TurnFail::SectionParse(err)) => {
                if !req.session.section_shape_nudged {
                    req.session.section_shape_nudged = true;
                    let _ = SECTION_SHAPE_NUDGE;
                    continue;
                }
                return Err(AgentError(format!(
                    "mini section parse failed after shape nudge: {}",
                    err.as_message()
                )));
            }
        }
    }
}

fn handle_overflow(
    req: &mut CompleteTurnRequest<'_>,
    counters: &LoopCounters,
    shrink_passes_used: &mut u32,
) -> Result<(), AgentError> {
    let max_shrink = BudgetScopeLayer::MiniShrinkPass
        .effective_max_attempts(req.config.max_shrink_passes, req.single_attempt);
    if *shrink_passes_used >= max_shrink {
        return Err(terminal_err(
            req,
            counters,
            MiniTerminalReason::ContextOverflow,
            "context overflow: shrink passes exhausted".into(),
        ));
    }
    *shrink_passes_used += 1;
    if let Some(event) = shrink_session_memory(req.session, *shrink_passes_used) {
        emit_prompt_shrink(req.trace, &event);
        return Ok(());
    }
    emit_prompt_shrink_stalled(req.trace);
    Err(terminal_err(
        req,
        counters,
        MiniTerminalReason::ContextOverflow,
        "context overflow: shrink stalled".into(),
    ))
}

fn terminal_err(
    req: &CompleteTurnRequest<'_>,
    counters: &LoopCounters,
    reason: MiniTerminalReason,
    msg: String,
) -> AgentError {
    let record = MiniTerminalRecord::new(
        reason,
        counters.http_turn_count,
        counters.bash_exec_count,
        MiniPhase::Investigate,
    );
    emit_terminal(req.trace, &record);
    AgentError(msg)
}

enum TurnFail {
    ContextOverflow,
    HttpExhausted(String),
    SectionParse(SectionParseError),
}

async fn complete_and_parse_turn(
    req: &mut CompleteTurnRequest<'_>,
) -> Result<ConsolidatedTurn, TurnFail> {
    let new_request = req.session.pending_new_request.clone().unwrap_or_default();
    let header = build_sticky_header(&req.config.mini_constraints, &req.session.llm_model_slug);
    let messages = assemble_session_messages(SessionAssemble {
        header: &header,
        study_act_cue: None,
        history: &req.session.history,
        previous_response: &req.session.previous_response,
        new_request: &new_request,
        section_nudge: req.session.section_shape_nudged,
    });

    crate::agent_phase::note_mini_llm_request();
    let t0 = Instant::now();
    let response = match complete_transport_with_retries(HttpRetryRequest {
        llm: req.llm,
        messages: &messages,
        max_transport_retries: req.config.max_transport_retries,
        single_attempt: req.single_attempt,
        timing: req.timing,
        trace: Some(req.trace),
    })
    .await
    {
        Ok(r) => r,
        Err(HttpCompletionError::ContextOverflow) => return Err(TurnFail::ContextOverflow),
        Err(HttpCompletionError::Exhausted(msg)) => return Err(TurnFail::HttpExhausted(msg)),
    };
    record_llm(
        req.timing,
        req.llm_phase.unwrap_or(TimingPhase::Implement),
        t0.elapsed(),
    );
    if let Some(ref usage) = response.usage {
        crate::run_timing::record_mini_http_cost(req.timing, usage);
    }
    req.trace.mini_llm_request(response.usage.as_ref());

    let parsed = parse_history_response(&response.content).map_err(TurnFail::SectionParse)?;
    Ok(ConsolidatedTurn {
        new_history: parsed.new_history,
        response: parsed.response,
        reasoning: response.reasoning,
    })
}
