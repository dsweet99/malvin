//! Driver types for the mini bash loop.

use super::loop_mock::LlmBackend;

pub struct LoopDriverConfig {
    pub max_bash_turns: u32,
    pub max_http_retries: u32,
    pub mini_constraints: &'static str,
}

pub struct LoopDriverSession {
    pub messages: Vec<malvin_mini::ChatMessage>,
    pub cwd: std::path::PathBuf,
}

pub struct LoopDriverOutcome {
    pub final_assistant_text: String,
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
}

#[cfg(test)]
mod tests {
    use super::{LoopDriverConfig, LoopDriverOutcome, LoopDriverRun, LoopDriverSession};

    #[test]
    fn loop_driver_config_and_outcome_types_are_constructible() {
        let config = LoopDriverConfig {
            max_bash_turns: 1,
            max_http_retries: 1,
            mini_constraints: "c",
        };
        assert_eq!(config.max_bash_turns, 1);
        let session = LoopDriverSession {
            messages: vec![],
            cwd: std::env::temp_dir(),
        };
        assert!(session.messages.is_empty());
        let outcome = LoopDriverOutcome {
            final_assistant_text: "done".into(),
        };
        assert_eq!(outcome.final_assistant_text, "done");
        let _: Option<LoopDriverRun> = None;
    }
}

#[cfg(test)]
#[path = "loop_types_test.rs"]
mod loop_types_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<LoopDriverConfig> = None;
        let _: Option<LoopDriverOutcome> = None;
        let _: Option<LoopDriverSession> = None;
    }
}
