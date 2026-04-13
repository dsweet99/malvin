/// ACP-backed agent with grounding-aligned session lifetimes.
///
/// Uses one long-lived **coder** session (implement → concerns → learn) and one **reviewer**
/// session per attempt (review + kpop), torn down after each pair.
pub struct AgentClient {
    pub model: String,
    pub io: AgentIoOptions,
    pub(crate) style_prompt_path: PathBuf,
    coder_session: Option<AcpSession>,
    /// When true, the next [`Self::run_coder_prompt`] prepends `.style/main.md` (first turn only).
    coder_style_on_next_prompt: bool,
    /// When set (e.g. `malvin code` orchestrator), LLM waits and retry backoff are recorded.
    pub(crate) timing: Option<std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
}
