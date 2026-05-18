// Spawn arguments split from [`session_types`](super::session_types) for dependency limits.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;

/// Arguments for spawning an [`AcpSession`](super::session_types::AcpSession).
#[allow(clippy::struct_excessive_bools)]
pub struct AcpSpawnArgs<'a> {
    pub cwd: &'a Path,
    pub bin_override: Option<&'a Path>,
    pub api_key: Option<&'a str>,
    pub auth_token: Option<&'a str>,
    pub rpc_timeout: Duration,
    pub acp_verbose: bool,
    pub george_acp_lane: Option<&'a str>,
    pub ui_idle_notify: Option<Arc<Notify>>,
    /// Passed through to `agent --model` when non-empty.
    pub model: Option<&'a str>,
    /// When true, passes `agent --force`.
    pub force: bool,
    /// When true, prints each trace line to stdout as it is written (live tee). Set from CLI tee mode.
    pub tee_trace_stdout: bool,
    /// When true, prints raw output without timestamps/prefixes (for raw `malvin do`).
    pub raw_output: bool,
    /// When true, raw/plain stdout includes thought chunks.
    pub show_thoughts_on_stdout: bool,
    /// When true, allows styled markdown on stdout for tagged trace lines (`malvin code` / `malvin kpop`).
    pub emit_stdout_markdown: bool,
    /// When set, each outgoing prompt appends timestamped lines to `prompts.log` under this directory.
    pub prompts_log_run_dir: Option<&'a Path>,
    /// When true, mirrors full outgoing prompt bodies to stdout and `prompts.log`; when false, name-only.
    pub log_full_outgoing_prompts: bool,
}
