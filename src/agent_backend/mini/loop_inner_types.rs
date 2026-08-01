//! Types for the inner bash-fence loop.

use crate::agent_backend::mini::fence_parser::BashFence;
use crate::agent_backend::mini::terminal::MiniTerminalReason;
use super::loop_mock::LlmBackend;
use super::loop_types::{LoopDriverConfig, LoopDriverSession};
use crate::run_timing::TimingPhase;

pub(crate) enum TurnAction {
    Done(MiniTerminalReason),
    RunBash(Vec<BashFence>),
}

pub(crate) enum LoopPhase {
    Investigate,
    WindDown,
}

pub(crate) struct LoopCounters {
    pub(crate) http_turn_count: u32,
    pub(crate) bash_exec_count: u32,
    pub(crate) investigate_http_turns: u32,
    pub(crate) had_bash_this_prompt: bool,
}

pub(crate) struct CompleteTurnRequest<'a> {
    pub(crate) llm: &'a LlmBackend,
    pub(crate) session: &'a mut LoopDriverSession,
    pub(crate) config: &'a LoopDriverConfig,
    pub(crate) trace: &'a crate::agent_backend::mini::trace::MiniTraceSink,
    pub(crate) timing: Option<&'a std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
    pub(crate) llm_phase: Option<TimingPhase>,
    pub(crate) single_attempt: bool,
}
