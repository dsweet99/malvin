//! Bash observation append for the inner bash-fence loop.

use std::time::Instant;

use malvin_mini::{ChatMessage, ChatRole};

use crate::agent_backend::mini::bash_adapter::{format_observation, run_bash_command, BashExecResult};
use crate::agent_backend::mini::fence_parser::BashFence;
use super::loop_inner_types::LoopCounters;
use super::loop_types::LoopDriverSession;
use crate::acp::AgentError;

pub(crate) struct BashObservationInput<'a> {
    pub(crate) session: &'a mut LoopDriverSession,
    pub(crate) trace: &'a crate::agent_backend::mini::trace::MiniTraceSink,
    pub(crate) transcript: &'a mut String,
    pub(crate) counters: &'a mut LoopCounters,
}

pub(crate) fn append_bash_observation(
    fences: &[BashFence],
    input: BashObservationInput<'_>,
) -> Result<(), AgentError> {
    let BashObservationInput {
        session,
        trace,
        transcript,
        counters,
    } = input;
    let mut results: Vec<BashExecResult> = Vec::new();
    for fence in fences {
        crate::agent_phase::note_mini_bash_exec();
        session.bash_commands_this_prompt.push(fence.command.clone());
        let t0 = Instant::now();
        let result = run_bash_command(
            session.cwd.as_path(),
            &fence.command,
            &session.llm_model_slug,
        )
        .map_err(AgentError)?;
        counters.bash_exec_count += 1;
        let elapsed = t0.elapsed();
        trace.mini_bash_exec(&fence.command, result.exit_code, elapsed, fence.comment.as_deref());
        crate::agent_phase::note_mini_bash_exec_done(result.exit_code, &fence.command);
        results.push(result);
    }
    let observation = format_observation(&results);
    session.messages.push(ChatMessage {
        role: ChatRole::User,
        content: observation,
    });
    if let Some(last) = session.messages.last() {
        transcript.push_str(&last.content);
        transcript.push('\n');
    }
    Ok(())
}
