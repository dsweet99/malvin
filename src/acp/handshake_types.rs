//! Handshake / stdio pipe bundle types for `agent acp`.
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::time::Duration;
use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio::sync::{Mutex, Notify};

use super::session_types::ResponseTx;

pub struct AcpHandshakeIo {
    pub stdin: Arc<Mutex<ChildStdin>>,
    pub pending: Arc<Mutex<HashMap<u64, ResponseTx>>>,
    pub reader_dead: Arc<AtomicBool>,
    pub next_id: Arc<AtomicU64>,
    pub busy: Arc<AtomicBool>,
    pub trace_writer: Arc<Mutex<Option<tokio::fs::File>>>,
    pub prompt_rpc_id: Arc<AtomicU64>,
    pub ui_idle_notify: Option<Arc<Notify>>,
}

pub struct AcpHandshakeSessionOpts {
    pub acp_verbose: bool,
    pub require_cursor_login_auth: bool,
    pub tee_trace_stdout: bool,
}

pub struct AcpChildStdout {
    pub child: Child,
    pub stdout: ChildStdout,
}

pub struct AcpHandshakeContinuation<'a> {
    pub cwd: &'a Path,
    pub rpc_timeout: Duration,
    pub session: AcpHandshakeSessionOpts,
}

#[test]
fn kiss_stringify_handshake_types() {
    let _ = stringify!(AcpHandshakeIo);
    let _ = stringify!(AcpHandshakeSessionOpts);
    let _ = stringify!(AcpChildStdout);
    let _ = stringify!(AcpHandshakeContinuation);
}
