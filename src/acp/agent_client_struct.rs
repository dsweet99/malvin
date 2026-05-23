use std::path::PathBuf;

use super::{AgentIoOptions, AcpSession};

/// ACP-backed agent with session-scoped coder and reviewer lifetimes.
///
/// In the **`malvin code`** orchestrator, one long-lived **coder** session spans `check_plan`
/// (unless skipped), `implement`, review prompts, optional `learn`, and `concerns` prompts that
/// run only after a review attempt fails to produce LGTM. Review fan-out is now prompt-driven:
/// the coder session runs `reviewers_spawn.md`, then `review_write.md`. KPOP is driven by
/// `run_kpop_flow` / `run_kpop_multiturn` / `run_kpop_flow_once`, not the review API.
pub struct AgentClient {
    pub model: String,
    pub io: AgentIoOptions,
    pub prompts_log_run_dir: Option<std::path::PathBuf>,
    pub(crate) style_prompt_path: PathBuf,
    pub(crate) coder_session: Option<AcpSession>,
    /// When true, the next [`Self::run_coder_prompt`] prepends injected repo style (first turn only).
    pub(crate) coder_style_on_next_prompt: bool,
    /// When set (e.g. `malvin code` orchestrator), LLM waits and retry backoff are recorded.
    pub(crate) timing: Option<std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
}
