use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use tokio::sync::Notify;

pub(super) fn acp_activity_state() -> (Arc<AtomicU64>, Arc<Notify>) {
    (Arc::new(AtomicU64::new(0)), Arc::new(Notify::new()))
}

#[cfg(unix)]
pub(super) const CAT_BIN: &str = "cat";

#[cfg(unix)]
use std::collections::HashMap;

#[cfg(unix)]
pub(super) fn incoming_permission_dispatch_plain<'a>(
    pending: &'a Arc<
        tokio::sync::Mutex<std::collections::HashMap<u64, crate::acp::ResponseTx>>,
    >,
    stdin: &'a Arc<tokio::sync::Mutex<tokio::process::ChildStdin>>,
    acp_activity_seq: &'a Arc<AtomicU64>,
    acp_activity_notify: &'a Arc<Notify>,
) -> crate::acp::IncomingLineDispatch<'a> {
    crate::acp::IncomingLineDispatch {
        pending,
        stdin,
        acp_activity_seq,
        acp_activity_notify,
        prompt_cleanup: None,
        acp_verbose: false,
        trace_jsonl: None,
    }
}

#[cfg(unix)]
pub(super) struct UnixCatIncoming {
    pub child: tokio::process::Child,
    pub stdin: Arc<tokio::sync::Mutex<tokio::process::ChildStdin>>,
    pub stdout: tokio::process::ChildStdout,
    pub pending: Arc<tokio::sync::Mutex<std::collections::HashMap<u64, crate::acp::ResponseTx>>>,
    pub acp_activity_seq: Arc<AtomicU64>,
    pub acp_activity_notify: Arc<Notify>,
}

#[cfg(unix)]
pub(super) async fn unix_cat_stdio_incoming(bin: &str) -> UnixCatIncoming {
    let pending = Arc::new(tokio::sync::Mutex::new(HashMap::new()));
    let (acp_activity_seq, acp_activity_notify) = acp_activity_state();
    let mut child =
        tokio::process::Command::new(crate::acp_test_unix_bin::unix_bin_with_fallback(bin))
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("spawn cat");
    let stdin = Arc::new(tokio::sync::Mutex::new(
        child.stdin.take().expect("stdin"),
    ));
    let stdout = child.stdout.take().expect("stdout");
    UnixCatIncoming {
        child,
        stdin,
        stdout,
        pending,
        acp_activity_seq,
        acp_activity_notify,
    }
}

#[cfg(unix)]
pub(super) async fn unix_true_exited_stdio_stdin_only() -> Arc<tokio::sync::Mutex<tokio::process::ChildStdin>> {
    let mut child =
        tokio::process::Command::new(crate::acp_test_unix_bin::unix_bin_with_fallback("true"))
            .stdin(std::process::Stdio::piped())
            .spawn()
            .expect("true");
    let stdin = Arc::new(tokio::sync::Mutex::new(
        child.stdin.take().expect("stdin"),
    ));
    let _ = child.wait().await;
    stdin
}

fn sleep_stdin_pipe_holder() -> (
    tokio::process::Child,
    Arc<tokio::sync::Mutex<tokio::process::ChildStdin>>,
) {
    let mut stdin_holder =
        tokio::process::Command::new(crate::acp_test_unix_bin::unix_bin_with_fallback("sleep"))
            .arg("1")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .expect("sleep");
    let stdin = Arc::new(tokio::sync::Mutex::new(
        stdin_holder.stdin.take().expect("stdin"),
    ));
    (stdin_holder, stdin)
}

fn idle_prompt_cleanup_bundle() -> (
    Arc<crate::acp::PromptRpcCleanup>,
    Arc<tokio::sync::Mutex<Option<crate::acp::PromptTraceWriter>>>,
) {
    use std::sync::atomic::AtomicBool;
    let trace_writer: Arc<tokio::sync::Mutex<Option<crate::acp::PromptTraceWriter>>> =
        Arc::new(tokio::sync::Mutex::new(None));
    let busy = Arc::new(AtomicBool::new(false));
    let prompt_rpc_id = Arc::new(AtomicU64::new(0));
    (
        Arc::new(crate::acp::PromptRpcCleanup {
            busy,
            trace_writer: trace_writer.clone(),
            prompt_rpc_id,
            idle_notify: None,
        }),
        trace_writer,
    )
}

async fn spawned_true_stdout_pending_wire() -> (
    tokio::process::Child,
    tokio::process::ChildStdout,
    Arc<tokio::sync::Mutex<std::collections::HashMap<u64, crate::acp::ResponseTx>>>,
    tokio::sync::oneshot::Receiver<Result<serde_json::Value, String>>,
) {
    use crate::acp_test_unix_bin::unix_bin_with_fallback;
    use std::collections::HashMap;
    use std::process::Stdio;
    use tokio::process::Command;
    use tokio::sync::oneshot;

    let mut child = Command::new(unix_bin_with_fallback("true"))
        .stdout(Stdio::piped())
        .spawn()
        .expect("true");
    let stdout = child.stdout.take().expect("stdout");
    let pending: Arc<tokio::sync::Mutex<HashMap<u64, crate::acp::ResponseTx>>> =
        Arc::new(tokio::sync::Mutex::new(HashMap::new()));
    let (tx, rx) = oneshot::channel();
    pending.lock().await.insert(7, tx);
    (child, stdout, pending, rx)
}

pub(super) struct ReaderTrueStdoutPendingEof {
    pub waiter: tokio::task::JoinHandle<()>,
    pub rx: tokio::sync::oneshot::Receiver<Result<serde_json::Value, String>>,
    pub stdout_child: tokio::process::Child,
    pub stdin_holder: tokio::process::Child,
}

pub(super) async fn assemble_true_stdout_pending_reader(
    memory_containment: crate::acp_memory_containment::AcpMemoryContainment,
) -> ReaderTrueStdoutPendingEof {
    use crate::acp::{ReaderSpawnArgs, spawn_acp_stdout_reader};

    let (child, stdout, pending, rx) = spawned_true_stdout_pending_wire().await;
    let (stdin_holder, stdin) = sleep_stdin_pipe_holder();
    let (acp_activity_seq, acp_activity_notify) = acp_activity_state();
    let reader_dead = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let (prompt_cleanup, trace_writer) = idle_prompt_cleanup_bundle();
    let waiter = spawn_acp_stdout_reader(ReaderSpawnArgs {
        stdout,
        pending: pending.clone(),
        stdin,
        acp_activity_seq,
        acp_activity_notify,
        reader_dead,
        trace_writer,
        prompt_cleanup,
        acp_verbose: false,
        trace_jsonl: None,
        tee_trace_stdout: false,
        memory_containment,
    });

    ReaderTrueStdoutPendingEof {
        waiter,
        rx,
        stdout_child: child,
        stdin_holder,
    }
}

pub(super) async fn spawn_reader_true_stdout_pending_eof(
    memory_containment: crate::acp_memory_containment::AcpMemoryContainment,
) -> ReaderTrueStdoutPendingEof {
    assemble_true_stdout_pending_reader(memory_containment).await
}
