type RpcWaitContext<'a> = (
    &'a Arc<AtomicU64>,
    &'a Arc<tokio::sync::Notify>,
    &'a Arc<Mutex<HashMap<u64, ResponseTx>>>,
    Option<u32>,
    &'a crate::acp_memory_containment::AcpMemoryContainment,
);

pub(crate) async fn rpc_request(n: RpcRequestNext<'_>) -> Result<Value, String> {
    let id = n
        .next_id
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    rpc_request_with_correlation_id(RpcOutgoing {
        io: n.io,
        id,
        method: n.method,
        params: n.params,
        rpc_timeout: n.rpc_timeout,
        child_pid: n.child_pid,
    })
    .await
}

pub(crate) async fn rpc_wait_response(args: RpcWaitArgs<'_>) -> Result<Value, String> {
    let mut rx = args.rx;
    let mut seen_activity = args
        .acp_activity_seq
        .load(std::sync::atomic::Ordering::SeqCst);
    loop {
        let activity = args.acp_activity_notify.notified();
        tokio::pin!(activity);
        let latest = args
            .acp_activity_seq
            .load(std::sync::atomic::Ordering::SeqCst);
        if latest != seen_activity {
            seen_activity = latest;
            continue;
        }
        tokio::select! {
            ready_recv = &mut rx => {
                return ready_recv.map_err(|_| {
                    "no ACP JSON-RPC reply; response channel closed (agent exit, stdout EOF, or request canceled)"
                        .to_string()
                })?;
            }
            () = &mut activity => {
                seen_activity = args
                    .acp_activity_seq
                    .load(std::sync::atomic::Ordering::SeqCst);
            }
        }
    }
}

#[cfg(test)]
async fn spawn_cat_rpc_stdio_pair() -> (
    tokio::process::Child,
    tokio::process::ChildStdout,
    Arc<Mutex<ChildStdin>>,
) {
    use std::process::Stdio;
    use tokio::process::Command;

    let mut child = Command::new("cat")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn cat");
    let stdout = child.stdout.take().expect("stdout");
    let stdin = Arc::new(Mutex::new(child.stdin.take().expect("stdin")));
    (child, stdout, stdin)
}

#[cfg(test)]
async fn read_first_stdout_line(stdout: tokio::process::ChildStdout) -> String {
    use std::time::Duration;
    use tokio::io::{AsyncBufReadExt, BufReader};

    let mut lines = BufReader::new(stdout).lines();
    let line = tokio::time::timeout(Duration::from_secs(2), lines.next_line())
        .await
        .expect("timeout")
        .expect("stdout read io")
        .expect("eof before newline");
    drop(lines);
    line
}

#[tokio::test]
async fn write_rpc_line_appends_flush_line_readable_on_child_stdout() {
    let (mut child, stdout, stdin) = spawn_cat_rpc_stdio_pair().await;
    write_rpc_line(&stdin, r#"{"jsonrpc":"2.0","id":1}"#, false, None)
        .await
        .expect("write_rpc_line");
    drop(stdin);
    let line = read_first_stdout_line(stdout).await;
    assert!(line.contains("jsonrpc"), "{}", line);
    let _ = child.wait().await;
}

include!("rpc_part2_health.inc");

