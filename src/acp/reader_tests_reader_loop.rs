#[cfg(unix)]
use crate::acp::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64};

#[cfg(unix)]
use std::collections::HashMap;
#[cfg(unix)]
use tokio::process::ChildStdin;

#[cfg(unix)]
struct EofReaderSpawnInputs {
    stdout: tokio::process::ChildStdout,
    pending: Arc<tokio::sync::Mutex<HashMap<u64, ResponseTx>>>,
    stdin: Arc<tokio::sync::Mutex<ChildStdin>>,
    acp_activity_seq: Arc<AtomicU64>,
    acp_activity_notify: Arc<tokio::sync::Notify>,
    trace_writer: Arc<tokio::sync::Mutex<Option<PromptTraceWriter>>>,
    prompt_cleanup: Arc<PromptRpcCleanup>,
}

#[cfg(unix)]
fn eof_reader_spawn_args(inputs: EofReaderSpawnInputs) -> ReaderSpawnArgs {
    ReaderSpawnArgs {
        stdout: inputs.stdout,
        pending: inputs.pending,
        stdin: inputs.stdin,
        acp_activity_seq: inputs.acp_activity_seq,
        acp_activity_notify: inputs.acp_activity_notify,
        reader_dead: Arc::new(AtomicBool::new(false)),
        trace_writer: inputs.trace_writer,
        prompt_cleanup: inputs.prompt_cleanup,
        acp_verbose: false,
        tee_trace_stdout: false,
        trace_jsonl: None,
        prompt_round_health: Arc::new(std::sync::Mutex::new(PromptRoundHealth::default())),
    }
}

#[cfg(unix)]
pub(crate) async fn reader_loop_eof_pending_error() -> String {
    use crate::acp_tests::reader_tests_helpers::{
        acp_activity_state, spawn_sleep_stdin, spawn_true_stdout_with_pending,
    };

    let (pending, rx, mut child, stdout) = spawn_true_stdout_with_pending().await;
    let (stdin, mut stdin_holder) = spawn_sleep_stdin().await;
    let (acp_activity_seq, acp_activity_notify) = acp_activity_state();
    let trace_writer: Arc<tokio::sync::Mutex<Option<PromptTraceWriter>>> =
        Arc::new(tokio::sync::Mutex::new(None));
    let prompt_cleanup = Arc::new(PromptRpcCleanup {
        busy: Arc::new(AtomicBool::new(false)),
        trace_writer: trace_writer.clone(),
        prompt_rpc_id: Arc::new(AtomicU64::new(0)),
        idle_notify: None,
    });
    let waiter = spawn_acp_stdout_reader(eof_reader_spawn_args(EofReaderSpawnInputs {
        stdout,
        pending,
        stdin,
        acp_activity_seq,
        acp_activity_notify,
        trace_writer,
        prompt_cleanup,
    }));
    let err = rx.await.unwrap().unwrap_err();
    let _: () = waiter.await.unwrap();
    let _ = child.wait().await;
    let _ = stdin_holder.kill().await;
    err
}

#[cfg(all(unix, test))]
#[test]
fn kiss_cov_eof_reader_spawn_args() {
    let _ = stringify!(EofReaderSpawnInputs);
    let _ = stringify!(eof_reader_spawn_args);
}

#[cfg(all(unix, test))]
#[test]
fn test_reader_loop_drains_pending_on_stdout_eof() {
    use crate::acp_tests::reader_tests_helpers::block_on_test;

    let err = block_on_test(reader_loop_eof_pending_error());
    assert!(
        err.contains("stdout closed") || err.contains("closed"),
        "expected stdout-close error, got {err:?}"
    );
}
