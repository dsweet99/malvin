//! Terminal emission helpers for the inner bash-fence loop.

use crate::agent_backend::mini::terminal::{
    MiniPhase, MiniTerminalReason, MiniTerminalRecord,
};
use crate::agent_backend::mini::trace_audit::emit_terminal;
use super::loop_types::{LoopDriverOutcome, LoopDriverSession};
use crate::acp::AgentError;

pub(crate) struct TerminalEmitCtx {
    pub reason: MiniTerminalReason,
    pub http_turn_count: u32,
    pub bash_exec_count: u32,
    pub phase_at_exit: MiniPhase,
}

pub(crate) fn persist_turn_memory(
    session: &mut LoopDriverSession,
    transcript: &mut String,
    new_history: &str,
    response: &str,
) {
    session.history = new_history.to_string();
    session.previous_response = response.to_string();
    session.pending_new_request = None;
    session.section_shape_nudged = false;
    transcript.push_str(response);
    transcript.push('\n');
}

pub(crate) fn finish_done_turn(
    trace: &crate::agent_backend::mini::trace::MiniTraceSink,
    assistant_text: &str,
    reasoning: Option<&str>,
    ctx: TerminalEmitCtx,
) -> LoopDriverOutcome {
    let record = MiniTerminalRecord::new(
        ctx.reason,
        ctx.http_turn_count,
        ctx.bash_exec_count,
        ctx.phase_at_exit,
    );
    emit_terminal(trace, &record);
    trace.mini_assistant_with_reasoning(assistant_text, reasoning);
    LoopDriverOutcome {
        final_assistant_text: assistant_text.to_string(),
        terminal: record,
    }
}

pub(crate) fn finish_exhausted(
    trace: &crate::agent_backend::mini::trace::MiniTraceSink,
    max_http_turns: u32,
    transcript: &str,
    ctx: TerminalEmitCtx,
) -> AgentError {
    let record = MiniTerminalRecord::new(
        ctx.reason,
        ctx.http_turn_count,
        ctx.bash_exec_count,
        ctx.phase_at_exit,
    );
    emit_terminal(trace, &record);
    exhausted_error(max_http_turns, transcript)
}

pub(crate) fn exhausted_error(max_http_turns: u32, transcript: &str) -> AgentError {
    AgentError(format!(
        "bash_loop: exhausted --mini-max-http-turns ({max_http_turns}) with partial transcript:\n{transcript}"
    ))
}
