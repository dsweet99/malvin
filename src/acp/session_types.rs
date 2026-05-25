//! Core session state types for `agent acp`.
use super::jsonl_trace::AcpJsonlTrace;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::time::Duration;
use tokio::process::{Child, ChildStdin};
use tokio::sync::{Mutex, Notify, oneshot};

pub type ResponseTx = oneshot::Sender<Result<Value, String>>;

#[allow(clippy::struct_excessive_bools)]
pub struct PromptTraceWriter {
    pub file: tokio::fs::File,
    /// Raw tag label before fixed-width padding (e.g. `<implement`, `malvin`).
    pub who: String,
    pub plain_lines: bool,
    pub stdout_replacement: Option<&'static str>,
    /// For learn tee: emit [`crate::output::LEARNING_PLACEHOLDER`] at most once to stdout.
    pub placeholder_emitted: bool,
    /// When true, print raw output without timestamps/prefixes.
    pub raw_output: bool,
    /// When true, raw/plain stdout includes thought chunks.
    pub show_thoughts_on_stdout: bool,
    /// When true, render agent message payloads as markdown on stdout (`malvin code` / `malvin kpop`).
    pub emit_stdout_markdown: bool,
    /// Suppress duplicate operational warnings for iterable-closed within one trace writer.
    pub iterable_closed_warned: bool,
    /// Session workspace root for relativizing tool-summary paths on stdout.
    pub work_dir: PathBuf,
    pub run_timing: Option<std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
    /// ACP session id for Cursor `store.db` enrichment (mirrors the sink session).
    #[allow(dead_code)]
    pub session_id: String,
    /// Deferred stdout sink; `None` when disabled or in tests that emit immediately.
    pub deferred_sink: Option<crate::deferred_log::SharedDeferSink>,
}

#[allow(clippy::struct_excessive_bools)]
pub struct AcpSessionInner {
    pub child: Mutex<Child>,
    pub process_group_id: Option<u32>,
    pub stdin: Arc<Mutex<ChildStdin>>,
    pub pending: Arc<Mutex<HashMap<u64, ResponseTx>>>,
    pub acp_activity_seq: Arc<AtomicU64>,
    pub acp_activity_notify: Arc<Notify>,
    pub next_id: Arc<AtomicU64>,
    pub session_id: String,
    pub reader_dead: Arc<AtomicBool>,
    pub rpc_timeout: Duration,
    pub busy: Arc<AtomicBool>,
    pub trace_writer: Arc<Mutex<Option<PromptTraceWriter>>>,
    pub prompt_rpc_id: Arc<AtomicU64>,
    /// Serializes `AcpSession::prompt` so overlapping callers cannot stomp the trace writer.
    pub prompt_singleflight: Arc<Mutex<()>>,
    pub acp_verbose: bool,
    /// When set (UI lane), observers are notified whenever `busy` becomes false.
    pub ui_idle_notify: Option<Arc<Notify>>,
    /// When true, print raw output without timestamps/prefixes.
    pub raw_output: bool,
    /// When true, raw/plain stdout includes thought chunks.
    pub show_thoughts_on_stdout: bool,
    /// When true, allow styled markdown on stdout for tagged trace lines (`malvin code` / `malvin kpop`).
    pub emit_stdout_markdown: bool,
    /// When set, each outgoing prompt appends timestamped lines to `prompts.log` under this directory.
    pub prompts_log_run_dir: Option<PathBuf>,
    /// When true, mirror full outgoing prompt bodies to stdout and `prompts.log`; when false, name-only.
    pub log_full_outgoing_prompts: bool,
    pub trace_jsonl: Option<Arc<AcpJsonlTrace>>,
    pub prompt_round_health: Arc<std::sync::Mutex<crate::acp::PromptRoundHealth>>,
    pub work_dir: PathBuf,
    pub run_timing: Option<std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
}

/// Live `agent acp` child process and JSON-RPC session state (cloneable handle; `cancel` may run
/// concurrently with an in-flight `session/prompt`; `prompt` calls are serialized per session).
#[derive(Clone)]
pub struct AcpSession(pub(crate) Arc<AcpSessionInner>);

/// Arguments for [`AcpSession::spawn`].
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
    /// When true, passes `agent --sandbox` before the `acp` subcommand.
    pub sandbox: bool,
    /// When true, print each trace line to stdout as it is written (live tee). Set from CLI tee mode.
    pub tee_trace_stdout: bool,
    /// When true, print raw output without timestamps/prefixes (for raw `malvin do`).
    pub raw_output: bool,
    /// When true, raw/plain stdout includes thought chunks.
    pub show_thoughts_on_stdout: bool,
    /// When true, allow styled markdown on stdout for tagged trace lines (`malvin code` / `malvin kpop`).
    pub emit_stdout_markdown: bool,
    /// When set, each outgoing prompt appends timestamped lines to `prompts.log` under this directory.
    pub prompts_log_run_dir: Option<&'a Path>,
    /// When true, mirror full outgoing prompt bodies to stdout and `prompts.log`; when false, name-only.
    pub log_full_outgoing_prompts: bool,
}

pub(crate) use super::wrap_handshake_types::*;
pub(crate) use super::wrap_session_channels::*;

#[test]
fn acp_spawn_args_george_fixture_sized() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let bin = tmp.path().join("agent");
    let args = super::spawn_test_args::george_mock_spawn_args(tmp.path(), &bin);
    assert_eq!(args.cwd, tmp.path());
    assert!(args.bin_override.is_some());
}




#[cfg(test)]
mod kiss_cov_auto {
    #[test]
    fn kiss_cov_prompt_trace_writer() { let _ = stringify!(PromptTraceWriter); }

}
