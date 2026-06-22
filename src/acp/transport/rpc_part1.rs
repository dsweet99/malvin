// JSON-RPC request/response over ACP stdio.

use crate::acp::rpc_part2::{rpc_wait_response, RpcWaitContext};
use crate::acp::rpc_part2_health::child_health_timeout_error;
use crate::acp::RpcWaitArgs;
use crate::acp::{note_acp_trace_activity, AcpJsonlTrace, ResponseTx};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::io::AsyncWriteExt;
use tokio::process::ChildStdin;
use tokio::sync::Mutex;
use tracing::info;

/// Shared stdio transport state for JSON-RPC to `agent acp`.
pub(crate) struct AcpStdioRpc {
    pub reader_dead: Arc<std::sync::atomic::AtomicBool>,
    pub stdin: Arc<Mutex<ChildStdin>>,
    pub pending: Arc<Mutex<HashMap<u64, ResponseTx>>>,
    pub acp_activity_seq: Arc<AtomicU64>,
    pub acp_activity_notify: Arc<tokio::sync::Notify>,
    pub acp_verbose: bool,
    pub trace_jsonl: Option<Arc<AcpJsonlTrace>>,
}

pub(crate) struct RpcLineWriteOpts<'a> {
    pub line: &'a str,
    pub acp_verbose: bool,
    pub trace_jsonl: Option<&'a AcpJsonlTrace>,
    pub activity: Option<(&'a Arc<AtomicU64>, &'a Arc<tokio::sync::Notify>)>,
}

pub(crate) async fn write_rpc_line(
    stdin: &Arc<Mutex<ChildStdin>>,
    opts: RpcLineWriteOpts<'_>,
) -> Result<(), String> {
    if opts.acp_verbose {
        info!(
            target: "malvin::acp::io",
            direction = "out",
            line = %opts.line,
            "acp message"
        );
    }
    if let Some(trace) = opts.trace_jsonl {
        trace.append_line("out", opts.line);
    }
    let mut guard = stdin.lock().await;
    guard
        .write_all(opts.line.as_bytes())
        .await
        .map_err(|e| format!("acp stdin write: {e}"))?;
    guard
        .write_all(b"\n")
        .await
        .map_err(|e| format!("acp stdin newline: {e}"))?;
    guard
        .flush()
        .await
        .map_err(|e| format!("acp stdin flush: {e}"))?;
    drop(guard);
    if let Some((seq, notify)) = opts.activity {
        note_acp_trace_activity(seq, notify);
    }
    Ok(())
}

/// One outbound JSON-RPC request (correlation id chosen by caller).
#[allow(dead_code)]
pub(crate) struct RpcOutgoing<'a> {
    pub io: &'a AcpStdioRpc,
    pub id: u64,
    pub method: &'a str,
    pub params: Value,
    pub rpc_timeout: std::time::Duration,
    pub child_pid: Option<u32>,
}

/// Next-id JSON-RPC request (`id` from [`AtomicU64`]).
#[allow(dead_code)]
pub(crate) struct RpcRequestNext<'a> {
    pub io: &'a AcpStdioRpc,
    pub next_id: &'a Arc<AtomicU64>,
    pub method: &'a str,
    pub params: Value,
    pub rpc_timeout: std::time::Duration,
    pub child_pid: Option<u32>,
}

pub(crate) async fn rpc_request_with_correlation_id(o: RpcOutgoing<'_>) -> Result<Value, String> {
    let io = o.io;
    if io.reader_dead.load(std::sync::atomic::Ordering::SeqCst) {
        return Err("acp session is dead".into());
    }
    let (tx, rx) = oneshot::channel();
    io.pending.lock().await.insert(o.id, tx);
    let req = json!({
        "jsonrpc": "2.0",
        "id": o.id,
        "method": o.method,
        "params": o.params
    });
    let line = match serde_json::to_string(&req) {
        Ok(l) => l,
        Err(e) => {
            io.pending.lock().await.remove(&o.id);
            return Err(e.to_string());
        }
    };
    if let Err(e) = write_rpc_line(
        &io.stdin,
        RpcLineWriteOpts {
            line: &line,
            acp_verbose: io.acp_verbose,
            trace_jsonl: io.trace_jsonl.as_deref(),
            activity: Some((&io.acp_activity_seq, &io.acp_activity_notify)),
        },
    )
    .await
    {
        io.pending.lock().await.remove(&o.id);
        return Err(e);
    }
    let args = RpcWaitArgs {
        _pending: &io.pending,
        acp_activity_seq: &io.acp_activity_seq,
        acp_activity_notify: &io.acp_activity_notify,
        _id: o.id,
        rx,
        child_pid: o.child_pid,
    };
    let wait_state = (
        &io.acp_activity_seq,
        &io.acp_activity_notify,
        &io.pending,
        o.child_pid,
    );
    rpc_wait_with_timeout(
        o.id,
        o.rpc_timeout,
        rpc_wait_response(args),
        wait_state,
    )
    .await
}

pub(crate) async fn rpc_wait_with_timeout(
    id: u64,
    timeout: std::time::Duration,
    wait: impl std::future::Future<Output = Result<Value, String>>,
    state: RpcWaitContext<'_>,
) -> Result<Value, String> {
    let (acp_activity_seq, acp_activity_notify, pending, child_pid) = state;
    tokio::pin!(wait);
    loop {
        tokio::select! {
            biased;
            ready_recv = &mut wait => {
                let result = ready_recv?;
                pending.lock().await.remove(&id);
                return Ok(result);
            }
            () = acp_activity_notify.notified() => {
            }
            () = tokio::time::sleep(timeout) => {
                let timeout_err = if let Some(child_pid) = child_pid {
                    let grace = crate::child_health::silence_grace_for_rpc_timeout(timeout);
                    let health = crate::child_health::evaluate_after_acp_silence(child_pid, grace);
                    tokio::pin!(health);
                    tokio::select! {
                        biased;
                        ready_recv = &mut wait => {
                            let result = ready_recv?;
                            pending.lock().await.remove(&id);
                            return Ok(result);
                        }
                        outcome = &mut health => {
                            match outcome {
                                crate::child_health::SilenceHealthOutcome::StillBusyExtendWait => {
                                    Ok(())
                                }
                                crate::child_health::SilenceHealthOutcome::AppearsHung => {
                                    let err = child_health_timeout_error(
                                        crate::child_health::SilenceHealthOutcome::AppearsHung,
                                    )
                                    .unwrap_or_else(|| {
                                        format!("acp request id {id} timed out after {timeout:?}")
                                    });
                                    Err(err)
                                }
                                other => {
                                    let err = child_health_timeout_error(other)
                                    .unwrap_or_else(|| {
                                        format!("acp request id {id} timed out after {timeout:?}")
                                    });
                                    Err(err)
                                }
                            }
                        }
                    }
                } else {
                    Err(format!("acp request id {id} timed out after {timeout:?}"))
                };
                if timeout_err.is_ok() {
                    continue;
                }
                pending.lock().await.remove(&id);
                return Err(timeout_err.expect_err("timeout outcome must be Err"));
            }
        }
    }
}

#[cfg(test)]
#[path = "rpc_part1_kiss_cov_test.rs"]
mod rpc_part1_kiss_cov_test;
#[cfg(test)]
#[path = "rpc_part1_test.rs"]
mod rpc_part1_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<AcpStdioRpc> = None;
        let _: Option<RpcLineWriteOpts> = None;
        let _: Option<RpcOutgoing> = None;
        let _: Option<RpcRequestNext> = None;
    }
}
