//! Driver types for the mini bash loop.

use super::loop_mock::LlmBackend;
use crate::agent_backend::mini::retry_fork::MiniRetryStrategy;
use crate::agent_backend::mini::terminal::MiniTerminalRecord;

pub struct LoopDriverConfig {
    pub max_http_turns: u32,
    pub max_bash_execs: u32,
    pub max_http_retries: u32,
    pub max_transport_retries: u32,
    pub max_shrink_passes: u32,
    pub mini_constraints: &'static str,
    /// When true, a fenceless reply without `MINI_DONE` is `FencelessPremature`.
    pub expects_investigation: bool,
}

pub struct LoopDriverSession {
    pub messages: Vec<malvin_mini::ChatMessage>,
    pub cwd: std::path::PathBuf,
    pub constraints_prepended: bool,
    pub bash_commands_this_prompt: Vec<String>,
    pub prompt_index: u32,
    /// Resolved `OpenRouter` model slug for this session (`MALVIN_LLM` in bash).
    pub llm_model_slug: String,
}

pub struct LoopDriverOutcome {
    pub final_assistant_text: String,
    pub terminal: MiniTerminalRecord,
}

pub struct LoopDriverRun<'a> {
    pub llm: &'a LlmBackend,
    pub session: &'a mut LoopDriverSession,
    pub user_prompt: &'a str,
    pub config: &'a LoopDriverConfig,
    pub trace: &'a crate::agent_backend::mini::trace::MiniTraceSink,
    pub timing: Option<&'a std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
    pub llm_phase: Option<crate::run_timing::TimingPhase>,
    pub single_attempt: bool,
    /// Gate-iteration attempt (1-based). Cumulative-transcript retries skip re-pushing the user prompt when > 1.
    pub gate_attempt: u32,
    pub retry_strategy: MiniRetryStrategy,
}

#[cfg(test)]
mod tests {
    use super::{LoopDriverConfig, LoopDriverOutcome, LoopDriverRun, LoopDriverSession};
    use crate::agent_backend::mini::terminal::{MiniPhase, MiniTerminalReason, MiniTerminalRecord};

    #[test]
    fn loop_driver_config_and_outcome_types_are_constructible() {
        let config = LoopDriverConfig {
            max_http_turns: 1,
            max_bash_execs: 128,
            max_http_retries: 1,
            max_transport_retries: 3,
            max_shrink_passes: 0,
            mini_constraints: "c",
            expects_investigation: false,
        };
        assert_eq!(config.max_http_turns, 1);
        let session = LoopDriverSession {
            messages: vec![],
            cwd: std::env::temp_dir(),
            constraints_prepended: false,
            bash_commands_this_prompt: vec![],
            prompt_index: 0,
            llm_model_slug: String::new(),
        };
        assert!(session.messages.is_empty());
        let outcome = LoopDriverOutcome {
            final_assistant_text: "done".into(),
            terminal: MiniTerminalRecord::new(
                MiniTerminalReason::FencelessComplete,
                1,
                0,
                MiniPhase::Terminal,
            ),
        };
        assert_eq!(outcome.final_assistant_text, "done");
        let _: Option<LoopDriverRun> = None;
    }
}
