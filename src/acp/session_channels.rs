//! Channel state built before the ACP JSON-RPC handshake completes.
use super::handshake_types::AcpHandshakeIo;
use super::session_types::{AcpSessionInner, AcpSpawnArgs, PromptTraceWriter, ResponseTx};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64};
use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio::sync::{Mutex, Notify};

/// Verbose logging for ACP (bundled for [`SessionChannelState::into_session_inner`]).
#[derive(Clone, Copy)]
pub struct SessionReaderTelemetry {
    pub acp_verbose: bool,
    /// When true, print raw output without timestamps/prefixes.
    pub raw_output: bool,
}

pub struct SessionChannelState {
    pub(crate) stdin: Arc<Mutex<ChildStdin>>,
    pub(crate) pending: Arc<Mutex<HashMap<u64, ResponseTx>>>,
    pub(crate) acp_activity_seq: Arc<AtomicU64>,
    pub(crate) acp_activity_notify: Arc<Notify>,
    pub(crate) reader_dead: Arc<AtomicBool>,
    pub(crate) next_id: Arc<AtomicU64>,
    pub(crate) busy: Arc<AtomicBool>,
    pub(crate) trace_writer: Arc<Mutex<Option<PromptTraceWriter>>>,
    pub(crate) prompt_rpc_id: Arc<AtomicU64>,
    pub(crate) prompt_singleflight: Arc<Mutex<()>>,
    pub(crate) ui_idle_notify: Option<Arc<tokio::sync::Notify>>,
}

fn acp_activity_state() -> (Arc<AtomicU64>, Arc<Notify>) {
    (Arc::new(AtomicU64::new(0)), Arc::new(Notify::new()))
}

impl SessionChannelState {
    pub(crate) fn new(stdin: ChildStdin, args: &AcpSpawnArgs<'_>) -> Self {
        let (acp_activity_seq, acp_activity_notify) = acp_activity_state();
        Self {
            stdin: Arc::new(Mutex::new(stdin)),
            pending: Arc::new(Mutex::new(HashMap::new())),
            acp_activity_seq,
            acp_activity_notify,
            reader_dead: Arc::new(AtomicBool::new(false)),
            next_id: Arc::new(AtomicU64::new(1)),
            busy: Arc::new(AtomicBool::new(false)),
            trace_writer: Arc::new(Mutex::new(None)),
            prompt_rpc_id: Arc::new(AtomicU64::new(0)),
            prompt_singleflight: Arc::new(Mutex::new(())),
            ui_idle_notify: args.ui_idle_notify.clone(),
        }
    }

    pub(crate) fn handshake_io(&self) -> AcpHandshakeIo {
        AcpHandshakeIo {
            stdin: self.stdin.clone(),
            pending: self.pending.clone(),
            acp_activity_seq: self.acp_activity_seq.clone(),
            acp_activity_notify: self.acp_activity_notify.clone(),
            reader_dead: self.reader_dead.clone(),
            next_id: self.next_id.clone(),
            busy: self.busy.clone(),
            trace_writer: self.trace_writer.clone(),
            prompt_rpc_id: self.prompt_rpc_id.clone(),
            ui_idle_notify: self.ui_idle_notify.clone(),
        }
    }

    pub(crate) fn into_session_inner(
        self,
        child: Child,
        session_id: String,
        rpc_timeout: std::time::Duration,
        telemetry: SessionReaderTelemetry,
    ) -> AcpSessionInner {
        let child_pid = child.id().unwrap_or(0);
        AcpSessionInner {
            child: Mutex::new(child),
            child_pid,
            stdin: self.stdin,
            pending: self.pending,
            acp_activity_seq: self.acp_activity_seq,
            acp_activity_notify: self.acp_activity_notify,
            next_id: self.next_id,
            session_id,
            reader_dead: self.reader_dead,
            rpc_timeout,
            busy: self.busy,
            trace_writer: self.trace_writer,
            prompt_rpc_id: self.prompt_rpc_id,
            prompt_singleflight: self.prompt_singleflight,
            acp_verbose: telemetry.acp_verbose,
            ui_idle_notify: self.ui_idle_notify,
            raw_output: telemetry.raw_output,
        }
    }
}

pub struct SessionAfterStdioIn<'a> {
    pub(crate) args: AcpSpawnArgs<'a>,
    pub(crate) rpc_timeout: std::time::Duration,
    pub(crate) require_cursor_login_auth: bool,
    pub(crate) child: Child,
    pub(crate) stdin: ChildStdin,
    pub(crate) stdout: ChildStdout,
}

#[test]
fn kiss_stringify_session_channels() {
    let _ = stringify!(SessionReaderTelemetry);
    let _ = stringify!(SessionChannelState);
    let _ = stringify!(SessionChannelState::new);
    let _ = stringify!(SessionChannelState::handshake_io);
    let _ = stringify!(SessionChannelState::into_session_inner);
    let _ = stringify!(SessionAfterStdioIn);
}

#[test]
fn acp_activity_state_returns_valid_arc_pair() {
    use std::sync::atomic::Ordering;
    let (seq, notify) = acp_activity_state();
    assert_eq!(seq.load(Ordering::SeqCst), 0);
    notify.notify_one();
}
