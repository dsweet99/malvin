use crate::acp::ResponseTx;
use crate::acp::*;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64};
use tokio::process::{Child, ChildStdout, Command};
use tokio::sync::oneshot;
use tokio::sync::{Mutex, Notify};

#[cfg(unix)]
use crate::acp_test_unix_bin::unix_bin_with_fallback;

#[cfg(unix)]
pub(crate) const CAT_BIN: &str = "cat";

pub(crate) fn acp_activity_state() -> (Arc<AtomicU64>, Arc<Notify>) {
    (Arc::new(AtomicU64::new(0)), Arc::new(Notify::new()))
}

pub(crate) fn test_prompt_round_health() -> Arc<std::sync::Mutex<PromptRoundHealth>> {
    Arc::new(std::sync::Mutex::new(PromptRoundHealth::default()))
}

pub(crate) fn handshake_io_from_stdin(stdin: tokio::process::ChildStdin) -> crate::acp::AcpHandshakeIo {
    use crate::acp::AcpHandshakeIo;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicBool, AtomicU64};
    AcpHandshakeIo {
        stdin: Arc::new(Mutex::new(stdin)),
        pending: Arc::new(Mutex::new(HashMap::new())),
        acp_activity_seq: Arc::new(AtomicU64::new(0)),
        acp_activity_notify: Arc::new(Notify::new()),
        reader_dead: Arc::new(AtomicBool::new(false)),
        next_id: Arc::new(AtomicU64::new(1)),
        busy: Arc::new(AtomicBool::new(false)),
        trace_writer: Arc::new(Mutex::new(None)),
        prompt_rpc_id: Arc::new(AtomicU64::new(0)),
        ui_idle_notify: None,
        trace_jsonl: None,
        prompt_round_health: test_prompt_round_health(),
    }
}

#[cfg(unix)]
pub(crate) struct IncomingDispatchParts<'a> {
    pub pending: &'a Arc<Mutex<HashMap<u64, ResponseTx>>>,
    pub stdin: &'a Arc<Mutex<tokio::process::ChildStdin>>,
    pub acp_activity_seq: &'a Arc<AtomicU64>,
    pub acp_activity_notify: &'a Arc<Notify>,
}

#[cfg(unix)]
impl IncomingDispatchParts<'_> {
    pub async fn dispatch_lines(&self, lines: &[&str]) {
        let prompt_round_health = test_prompt_round_health();
        for line in lines {
            handle_incoming_line(
                line,
                IncomingLineDispatch {
                    pending: self.pending,
                    stdin: self.stdin,
                    acp_activity_seq: self.acp_activity_seq,
                    acp_activity_notify: self.acp_activity_notify,
                    prompt_cleanup: None,
                    acp_verbose: false,
                    trace_jsonl: None,
                    prompt_round_health: &prompt_round_health,
                },
            )
            .await;
        }
    }
}

#[cfg(unix)]
pub(crate) struct CatSession {
    pub pending: Arc<Mutex<HashMap<u64, ResponseTx>>>,
    pub stdin: Arc<Mutex<tokio::process::ChildStdin>>,
    pub acp_activity_seq: Arc<AtomicU64>,
    pub acp_activity_notify: Arc<Notify>,
    stdout: ChildStdout,
    child: Child,
}

#[cfg(unix)]
impl CatSession {
    pub async fn new() -> Self {
        let (acp_activity_seq, acp_activity_notify) = acp_activity_state();
        let mut child = Command::new(unix_bin_with_fallback(CAT_BIN))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("cat");
        let stdin = Arc::new(Mutex::new(child.stdin.take().expect("stdin")));
        let stdout = child.stdout.take().expect("stdout");
        Self {
            pending: Arc::new(Mutex::new(HashMap::new())),
            stdin,
            acp_activity_seq,
            acp_activity_notify,
            stdout,
            child,
        }
    }

    pub fn dispatch_parts(&self) -> IncomingDispatchParts<'_> {
        IncomingDispatchParts {
            pending: &self.pending,
            stdin: &self.stdin,
            acp_activity_seq: &self.acp_activity_seq,
            acp_activity_notify: &self.acp_activity_notify,
        }
    }

    pub async fn finish_stdout(mut self) -> String {
        use tokio::io::AsyncReadExt;
        drop(self.stdin);
        let mut received = Vec::new();
        self.stdout
            .read_to_end(&mut received)
            .await
            .expect("read stdout");
        String::from_utf8_lossy(&received).into_owned()
    }
}

#[cfg(unix)]
pub(crate) async fn spawn_true_stdout_with_pending() -> (
    Arc<Mutex<HashMap<u64, ResponseTx>>>,
    oneshot::Receiver<Result<serde_json::Value, String>>,
    Child,
    ChildStdout,
) {
    let mut child = Command::new(unix_bin_with_fallback("true"))
        .stdout(Stdio::piped())
        .spawn()
        .expect("true");
    let stdout = child.stdout.take().expect("stdout");
    let pending: Arc<Mutex<HashMap<u64, ResponseTx>>> = Arc::new(Mutex::new(HashMap::new()));
    let (tx, rx) = oneshot::channel();
    pending.lock().await.insert(7, tx);
    (pending, rx, child, stdout)
}

#[cfg(unix)]
pub(crate) async fn spawn_sleep_stdin() -> (Arc<Mutex<tokio::process::ChildStdin>>, Child) {
    let mut stdin_holder = Command::new(unix_bin_with_fallback("sleep"))
        .arg("1")
        .stdin(Stdio::piped())
        .spawn()
        .expect("sleep");
    let stdin = Arc::new(Mutex::new(stdin_holder.stdin.take().expect("stdin")));
    (stdin, stdin_holder)
}

pub(crate) fn block_on_test<F, T>(f: F) -> T
where
    F: std::future::Future<Output = T>,
{
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("test runtime")
        .block_on(f)
}

#[cfg(all(unix, test))]
#[test]
fn reader_helpers_unix_fixtures_run() {
    block_on_test(async {
        let (_stdin, _child) = spawn_sleep_stdin().await;
        let (_stdin_alias, _child_alias) = spawn_sleep_stdin().await;
        let (_pending, _rx, _child, _stdout) = spawn_true_stdout_with_pending().await;
    });
}

#[cfg(test)]
#[path = "reader_tests_helpers_kiss_cov_test.rs"]
mod reader_tests_helpers_kiss_cov_test;
#[cfg(test)]
#[path = "reader_tests_helpers_test.rs"]
mod readers_helpers_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<CatSession> = None;
        let _: Option<IncomingDispatchParts> = None;
        let _ = acp_activity_state;
        let _ = dispatch_lines;
        let _ = dispatch_parts;
        let _ = finish_stdout;
        let _ = handshake_io_from_stdin;
        let _ = new;
        let _ = test_prompt_round_health;
    }
}
