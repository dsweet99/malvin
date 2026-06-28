//! HTTP completion and context-recovery for the inner bash-fence loop.

use std::time::Instant;

use malvin_mini::CompletionResponse;

use crate::agent_backend::mini::context_recovery::shrink_one_whole_message;
use crate::agent_backend::mini::terminal::{
    MiniPhase, MiniTerminalReason, MiniTerminalRecord,
};
use crate::agent_backend::mini::trace_audit::{
    emit_prompt_shrink, emit_prompt_shrink_stalled, emit_terminal,
};
use super::loop_http::{complete_with_http_retries, HttpCompletionError, HttpRetryRequest};
use super::loop_inner_types::{CompleteTurnRequest, LoopCounters};
use crate::acp::AgentError;
use crate::run_timing::{record_llm, TimingPhase};

pub(crate) async fn complete_turn_with_recovery(
    req: &mut CompleteTurnRequest<'_>,
    counters: &LoopCounters,
) -> Result<CompletionResponse, AgentError> {
    let mut shrink_passes_used = 0_u32;
    loop {
        let turn_req = CompleteTurnRequest {
            llm: req.llm,
            session: req.session,
            config: req.config,
            trace: req.trace,
            timing: req.timing,
            llm_phase: req.llm_phase,
            single_attempt: req.single_attempt,
        };
        match complete_turn(turn_req).await {
            Ok(r) => return Ok(r),
            Err(HttpCompletionError::ContextOverflow) => {
                if shrink_passes_used >= req.config.max_shrink_passes {
                    let record = MiniTerminalRecord::new(
                        MiniTerminalReason::ContextOverflow,
                        counters.http_turn_count,
                        counters.bash_exec_count,
                        MiniPhase::Investigate,
                    );
                    emit_terminal(req.trace, &record);
                    return Err(AgentError("context overflow: shrink passes exhausted".into()));
                }
                shrink_passes_used += 1;
                if let Some(event) = shrink_one_whole_message(req.session, shrink_passes_used) {
                    emit_prompt_shrink(req.trace, &event);
                } else {
                    emit_prompt_shrink_stalled(req.trace);
                    let record = MiniTerminalRecord::new(
                        MiniTerminalReason::ContextOverflow,
                        counters.http_turn_count,
                        counters.bash_exec_count,
                        MiniPhase::Investigate,
                    );
                    emit_terminal(req.trace, &record);
                    return Err(AgentError("context overflow: shrink stalled".into()));
                }
            }
            Err(HttpCompletionError::Exhausted(msg)) => {
                let record = MiniTerminalRecord::new(
                    MiniTerminalReason::HttpRetryExhausted,
                    counters.http_turn_count,
                    counters.bash_exec_count,
                    MiniPhase::Investigate,
                );
                emit_terminal(req.trace, &record);
                return Err(AgentError(msg));
            }
        }
    }
}

async fn complete_turn(req: CompleteTurnRequest<'_>) -> Result<CompletionResponse, HttpCompletionError> {
    let CompleteTurnRequest {
        llm,
        session,
        config,
        trace,
        timing,
        llm_phase,
        single_attempt,
    } = req;
    crate::agent_phase::note_mini_llm_request();
    let t0 = Instant::now();
    let response = complete_with_http_retries(HttpRetryRequest {
        llm,
        messages: &session.messages,
        max_transport_retries: config.max_transport_retries,
        single_attempt,
        timing,
        trace: Some(trace),
    })
    .await?;
    record_llm(timing, llm_phase.unwrap_or(TimingPhase::Implement), t0.elapsed());
    if let Some(ref usage) = response.usage {
        crate::run_timing::record_mini_http_cost(timing, usage);
    }
    trace.mini_llm_request(response.usage.as_ref());
    Ok(response)
}
