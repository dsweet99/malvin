//! Agent interface surface: malvin orchestration → Mini or cursor-agent.
//!
//! The shared API is [`crate::agent_backend::AgentBackend`] method names
//! (`ensure_authenticated`, `begin_coder_session`, `run_coder_prompt`, …).
//! Kiss forbids trait definitions here; the enum is the Agent interface.

use crate::acp::CoderPromptOptions;

/// Options for a single agent prompt call.
#[derive(Debug, Clone, Copy, Default)]
pub struct PromptOptions {
    /// When true, the agent does not retry failed prompts; caller owns gates/KPop retries.
    pub single_attempt: bool,
}

impl PromptOptions {
    #[must_use]
    pub const fn from_coder(opts: &CoderPromptOptions<'_>) -> Self {
        Self {
            single_attempt: opts.single_attempt,
        }
    }
}

#[cfg(test)]
#[path = "kiss_coverage.rs"]
mod kiss_coverage;
