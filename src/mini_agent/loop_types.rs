//! Driver types for the mini bash loop.

use super::loop_mock::LlmBackend;
use crate::mini_agent::retry_fork::MiniRetryStrategy;
use crate::mini_agent::terminal::MiniTerminalRecord;

pub struct LoopDriverConfig {
    pub max_http_turns: u32,
    pub max_bash_execs: u32,
    pub max_http_retries: u32,
    pub max_transport_retries: u32,
    pub max_shrink_passes: u32,
    pub mini_constraints: String,
    /// When true, a fenceless reply without `MINI_DONE` is `FencelessPremature`.
    pub expects_investigation: bool,
}

/// Durable mini conversation state: chat-state History + Previous RESPONSE body.
pub struct LoopDriverSession {
    pub history: String,
    pub previous_response: String,
    /// Event that triggers the next consolidate (user text, bash obs, divergence).
    pub pending_new_request: Option<String>,
    pub cwd: std::path::PathBuf,
    pub bash_commands_this_prompt: Vec<String>,
    pub prompt_index: u32,
    /// Resolved `OpenRouter` model slug for this session (`MALVIN_LLM` in bash).
    pub llm_model_slug: String,
    /// One section-shape nudge already used for the current pending request.
    pub section_shape_nudged: bool,
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
    pub trace: &'a crate::mini_agent::trace::MiniTraceSink,
    pub timing: Option<&'a std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
    pub llm_phase: Option<crate::run_timing::TimingPhase>,
    pub single_attempt: bool,
    /// Gate-iteration attempt (1-based). Cumulative retries skip re-setting the user prompt when > 1
    /// if a divergence New request is already pending.
    pub gate_attempt: u32,
    pub retry_strategy: MiniRetryStrategy,
}

#[cfg(test)]
mod tests {
    use super::{LoopDriverConfig, LoopDriverOutcome, LoopDriverRun, LoopDriverSession};
    use crate::mini_agent::terminal::{MiniPhase, MiniTerminalReason, MiniTerminalRecord};

    #[test]
    fn loop_driver_config_and_outcome_types_are_constructible() {
        let config = LoopDriverConfig {
            max_http_turns: 1,
            max_bash_execs: 128,
            max_http_retries: 1,
            max_transport_retries: 3,
            max_shrink_passes: 0,
            mini_constraints: "c".into(),
            expects_investigation: false,
        };
        assert_eq!(config.max_http_turns, 1);
        let session = LoopDriverSession {
            history: String::new(),
            previous_response: String::new(),
            pending_new_request: None,
            cwd: std::env::temp_dir(),
            bash_commands_this_prompt: vec![],
            prompt_index: 0,
            llm_model_slug: String::new(),
            section_shape_nudged: false,
        };
        assert!(session.history.is_empty());
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
