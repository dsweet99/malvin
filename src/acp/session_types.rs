//! Core session state types for `agent acp`.
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::time::Duration;
use tokio::process::{Child, ChildStdin};
use tokio::sync::{Mutex, Notify, oneshot};

pub type ResponseTx = oneshot::Sender<Result<Value, String>>;

pub struct AcpSessionInner {
    pub child: Mutex<Child>,
    pub stdin: Arc<Mutex<ChildStdin>>,
    pub pending: Arc<Mutex<HashMap<u64, ResponseTx>>>,
    pub next_id: Arc<AtomicU64>,
    pub session_id: String,
    pub reader_dead: Arc<AtomicBool>,
    pub rpc_timeout: Duration,
    pub busy: Arc<AtomicBool>,
    pub trace_writer: Arc<Mutex<Option<tokio::fs::File>>>,
    pub prompt_rpc_id: Arc<AtomicU64>,
    /// Serializes `AcpSession::prompt` so overlapping callers cannot stomp the trace writer.
    pub prompt_singleflight: Arc<Mutex<()>>,
    pub acp_verbose: bool,
    /// When set (UI lane), observers are notified whenever `busy` becomes false.
    pub ui_idle_notify: Option<Arc<Notify>>,
}

/// Live `agent acp` child process and JSON-RPC session state (cloneable handle; `cancel` may run
/// concurrently with an in-flight `session/prompt`; `prompt` calls are serialized per session).
#[derive(Clone)]
pub struct AcpSession(pub(crate) Arc<AcpSessionInner>);

/// Arguments for [`AcpSession::spawn`].
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
}

#[test]
fn kiss_stringify_session_types() {
    let _ = stringify!(ResponseTx);
    let _ = stringify!(AcpSessionInner);
    let _ = stringify!(AcpSession);
    let _ = stringify!(AcpSpawnArgs);
}
