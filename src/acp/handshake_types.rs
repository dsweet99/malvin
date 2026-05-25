use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::time::Duration;

use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio::sync::{Mutex, Notify};

use super::super::jsonl_trace::AcpJsonlTrace;
use super::super::prompt_round_health::PromptRoundHealth;
use super::super::session_types::{PromptTraceWriter, ResponseTx};

pub struct AcpHandshakeIo {
    pub stdin: Arc<Mutex<ChildStdin>>,
    pub pending: Arc<Mutex<HashMap<u64, ResponseTx>>>,
    pub acp_activity_seq: Arc<AtomicU64>,
    pub acp_activity_notify: Arc<Notify>,
    pub reader_dead: Arc<AtomicBool>,
    pub next_id: Arc<AtomicU64>,
    pub busy: Arc<AtomicBool>,
    pub trace_writer: Arc<Mutex<Option<PromptTraceWriter>>>,
    pub prompt_rpc_id: Arc<AtomicU64>,
    pub ui_idle_notify: Option<Arc<Notify>>,
    pub trace_jsonl: Option<Arc<AcpJsonlTrace>>,
    pub prompt_round_health: Arc<std::sync::Mutex<PromptRoundHealth>>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handshake_opts_and_continuation_fixture() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let session_opts = AcpHandshakeSessionOpts {
            acp_verbose: false,
            require_cursor_login_auth: false,
            tee_trace_stdout: false,
        };
        let _cont = AcpHandshakeContinuation {
            cwd: tmp.path(),
            rpc_timeout: Duration::from_secs(10),
            session: session_opts,
        };
    }

    #[tokio::test]
    async fn handshake_io_constructed_like_channel_placeholder() {
        let mut child = tokio::process::Command::new("cat")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .spawn()
            .expect("spawn cat");
        let stdin = child.stdin.take().expect("stdin");
        let io = crate::acp_tests::reader_tests_helpers::handshake_io_from_stdin(stdin);
        assert!(io
            .prompt_round_health
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_empty());
        child.kill().await.ok();
        let _ = child.wait().await;
    }

    #[tokio::test]
    async fn acp_handshake_child_stdout_placeholder() {
        let mut child = tokio::process::Command::new("cat")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("spawn cat");
        let stdout = child.stdout.take().expect("stdout");
        let mut bundle = AcpChildStdout { child, stdout };
        bundle.child.kill().await.ok();
        let _ = bundle.child.wait().await;
    }
}
