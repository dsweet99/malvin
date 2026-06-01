use super::{AgentIoOptions, AcpSession};

/// ACP-backed agent with session-scoped coder lifetimes.
///
/// Bug remediation and summary phases run coder prompts on one long-lived session.
/// `KPop` is driven by `run_kpop_flow` / `run_kpop_multiturn` / `run_kpop_flow_once`.
pub struct AgentClient {
    pub model: String,
    pub io: AgentIoOptions,
    pub prompts_log_run_dir: Option<std::path::PathBuf>,
    pub(crate) coder_session: Option<AcpSession>,
    /// Coder session cwd from the last successful [`super::AgentClient::begin_coder_session`].
    pub(crate) coder_session_cwd: Option<std::path::PathBuf>,
    /// Bounded attempts per ACP spawn or `session/prompt` (from `--max-acp-retries`).
    pub(crate) max_acp_retries: u32,
    /// When set (e.g. `malvin code` orchestrator), LLM waits and retry backoff are recorded.
    pub(crate) timing: Option<std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
}
