//! Unified agent backend dispatch.

use std::path::{Path, PathBuf};

use crate::acp::{AgentClient, AgentError, AuthError, CoderPromptOptions};
use crate::cursor_sdk::CursorSdkClient;
use crate::mini_agent::MiniAgentClient;

#[allow(clippy::large_enum_variant)]
pub enum AgentBackend {
    Acp(AgentClient),
    CursorSdk(CursorSdkClient),
    Mini(MiniAgentClient),
}

impl AgentBackend {
    pub fn ensure_authenticated(&self) -> Result<(), AuthError> {
        match self {
            Self::Acp(c) => c.ensure_authenticated(),
            Self::CursorSdk(c) => c.ensure_authenticated(),
            Self::Mini(c) => c.ensure_authenticated(),
        }
    }

    #[must_use]
    pub const fn has_open_coder_session(&self) -> bool {
        match self {
            Self::Acp(c) => c.has_open_coder_session(),
            Self::CursorSdk(c) => c.has_open_coder_session(),
            Self::Mini(c) => c.has_open_coder_session(),
        }
    }

    /// Cursor SDK Node bridge: keep one process across Continues / gate turns,
    /// refreshing it when [`super::agent_backend_ensure_coder_session`] sees it is stale.
    /// ACP/Mini still end between outer Continues / gate turns (fresh agent + memory headroom).
    #[must_use]
    pub const fn keeps_coder_session_for_process_life(&self) -> bool {
        matches!(self, Self::CursorSdk(_))
    }

    pub async fn begin_coder_session(&mut self, cwd: &Path) -> Result<(), AgentError> {
        match self {
            Self::Acp(c) => c.begin_coder_session(cwd).await,
            Self::CursorSdk(c) => c.begin_coder_session(cwd).await,
            Self::Mini(c) => c.begin_coder_session(cwd).await,
        }
    }

    pub async fn run_coder_prompt(
        &mut self,
        prompt: &str,
        log_path: &Path,
        who: &str,
        opts: CoderPromptOptions<'_>,
    ) -> Result<(), AgentError> {
        match self {
            Self::Acp(c) => c.run_coder_prompt(prompt, log_path, who, opts).await,
            Self::CursorSdk(c) => c.run_coder_prompt(prompt, log_path, who, opts).await,
            Self::Mini(c) => c.run_coder_prompt(prompt, log_path, who, opts).await,
        }
    }

    pub async fn end_coder_session(&mut self) -> Result<(), AgentError> {
        match self {
            Self::Acp(c) => c.end_coder_session().await,
            Self::CursorSdk(c) => c.end_coder_session().await,
            Self::Mini(c) => c.end_coder_session().await,
        }
    }

    #[must_use]
    pub fn last_coder_prompt_agent_response(&self) -> Option<String> {
        match self {
            Self::Acp(c) => c.last_coder_prompt_agent_response(),
            Self::CursorSdk(c) => c.last_coder_prompt_agent_response(),
            Self::Mini(c) => c.last_coder_prompt_agent_response(),
        }
    }

    #[must_use]
    pub const fn max_acp_retries(&self) -> u32 {
        match self {
            Self::Acp(c) => c.max_acp_retries,
            Self::CursorSdk(c) => c.max_acp_retries,
            Self::Mini(c) => c.max_acp_retries(),
        }
    }

    #[must_use]
    pub const fn prompts_log_run_dir(&self) -> Option<&PathBuf> {
        match self {
            Self::Acp(c) => c.prompts_log_run_dir.as_ref(),
            Self::CursorSdk(c) => c.prompts_log_run_dir.as_ref(),
            Self::Mini(c) => c.trace_run_dir.as_ref(),
        }
    }

    pub fn set_prompts_log_run_dir(&mut self, dir: Option<PathBuf>) {
        match self {
            Self::Acp(c) => c.prompts_log_run_dir = dir,
            Self::CursorSdk(c) => c.prompts_log_run_dir = dir,
            Self::Mini(c) => c.trace_run_dir = dir,
        }
    }
}
