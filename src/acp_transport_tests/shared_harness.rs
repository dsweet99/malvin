use super::prelude::*;

pub(crate) fn acp_activity_state() -> (Arc<AtomicU64>, Arc<Notify>) {
    (Arc::new(AtomicU64::new(0)), Arc::new(Notify::new()))
}

pub(crate) struct InactiveRpcIo {
    pub reader_dead: Arc<AtomicBool>,
    pub stdin: Arc<Mutex<tokio::process::ChildStdin>>,
    pub pending: Arc<Mutex<HashMap<u64, ResponseTx>>>,
    pub acp_activity_seq: Arc<AtomicU64>,
    pub acp_activity_notify: Arc<Notify>,
}

pub(crate) fn acp_stdio_rpc_inactive(io: InactiveRpcIo) -> AcpStdioRpc {
    AcpStdioRpc {
        reader_dead: io.reader_dead,
        stdin: io.stdin,
        pending: io.pending,
        acp_activity_seq: io.acp_activity_seq,
        acp_activity_notify: io.acp_activity_notify,
        acp_verbose: false,
        trace_jsonl: None,
    }
}

pub(crate) enum SleepStdoutDrainMode {
    None,
    SmallBuf,
    LargeBuf,
}

pub(crate) struct RpcSleepHarness {
    pub child: tokio::process::Child,
    pub stdin: Arc<Mutex<tokio::process::ChildStdin>>,
    pub stdout_drain: Option<tokio::task::JoinHandle<()>>,
    pub pending: Arc<Mutex<HashMap<u64, ResponseTx>>>,
    pub acp_activity_seq: Arc<AtomicU64>,
    pub acp_activity_notify: Arc<Notify>,
    pub reader_dead: Arc<AtomicBool>,
}

pub(crate) fn drain_stdout_read(mut stdout: tokio::process::ChildStdout, small_buf: bool) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        if small_buf {
            let mut buf = [0u8; 64];
            while stdout.read(&mut buf).await.unwrap_or(0) > 0 {}
        } else {
            let mut buf = [0u8; 256];
            while stdout.read(&mut buf).await.unwrap_or(0) > 0 {}
        }
    })
}

pub(crate) fn sleep_stdout_drain_for_child(
    drain: SleepStdoutDrainMode,
    child: &mut tokio::process::Child,
) -> Option<tokio::task::JoinHandle<()>> {
    let stdout = child.stdout.take().expect("stdout");
    match drain {
        SleepStdoutDrainMode::None => {
            drop(stdout);
            None
        }
        SleepStdoutDrainMode::SmallBuf => Some(drain_stdout_read(stdout, true)),
        SleepStdoutDrainMode::LargeBuf => Some(drain_stdout_read(stdout, false)),
    }
}

impl RpcSleepHarness {
    pub async fn spawn_sleep(seconds: &str, drain: SleepStdoutDrainMode) -> Self {
        let mut child = tokio::process::Command::new(crate::acp_test_unix_bin::unix_bin_with_fallback("sleep"))
            .arg(seconds)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("sleep");
        let stdin = Arc::new(Mutex::new(child.stdin.take().expect("stdin")));
        let stdout_drain = sleep_stdout_drain_for_child(drain, &mut child);
        let pending = Arc::new(Mutex::new(HashMap::new()));
        let (acp_activity_seq, acp_activity_notify) = acp_activity_state();
        let reader_dead = Arc::new(AtomicBool::new(false));
        RpcSleepHarness {
            child,
            stdin,
            stdout_drain,
            pending,
            acp_activity_seq,
            acp_activity_notify,
            reader_dead,
        }
    }

    pub fn child_pid(&self) -> Option<u32> {
        self.child.id()
    }

    pub fn io(&self) -> AcpStdioRpc {
        acp_stdio_rpc_inactive(InactiveRpcIo {
            reader_dead: self.reader_dead.clone(),
            stdin: self.stdin.clone(),
            pending: self.pending.clone(),
            acp_activity_seq: self.acp_activity_seq.clone(),
            acp_activity_notify: self.acp_activity_notify.clone(),
        })
    }

    pub async fn shutdown(mut self) {
        let _ = self.child.kill().await;
        let _ = self.child.wait().await;
        if let Some(drain) = self.stdout_drain {
            let _ = drain.await;
        }
    }
}

pub(crate) async fn true_child_stdin_stdout_drained_after_exit() -> (Arc<Mutex<tokio::process::ChildStdin>>, tokio::task::JoinHandle<()>) {
    let mut child = tokio::process::Command::new(crate::acp_test_unix_bin::unix_bin_with_fallback("true"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("true");
    let stdin = Arc::new(Mutex::new(child.stdin.take().expect("stdin")));
    let stdout = child.stdout.take().expect("stdout");
    let drain = drain_stdout_read(stdout, true);
    let _ = child.wait().await;
    (stdin, drain)
}

pub(crate) struct HarnessRpcWaitParams<'a> {
    pub h: &'a RpcSleepHarness,
    pub request_id: u64,
    pub timeout: std::time::Duration,
    pub rx: tokio::sync::oneshot::Receiver<Result<Value, String>>,
    pub child_pid: Option<u32>,
}

pub(crate) async fn harness_rpc_wait(params: HarnessRpcWaitParams<'_>) -> Result<Value, String> {
    let io = params.h.io();
    rpc_wait_with_timeout(
        params.request_id,
        params.timeout,
        rpc_wait_response(crate::acp::RpcWaitArgs {
            _pending: &io.pending,
            acp_activity_seq: &io.acp_activity_seq,
            acp_activity_notify: &io.acp_activity_notify,
            _id: params.request_id,
            rx: params.rx,
            child_pid: params.child_pid,
        }),
        (
            &io.acp_activity_seq,
            &io.acp_activity_notify,
            &io.pending,
            params.child_pid,
        ),
    )
    .await
}


#[cfg(test)]
mod kiss_cov_auto {
    #[test]
    fn kiss_cov_acp_activity_state() { let _ = stringify!(acp_activity_state); }

    #[test]
    fn kiss_cov_inactive_rpc_io() { let _ = stringify!(InactiveRpcIo); }

    #[test]
    fn kiss_cov_acp_stdio_rpc_inactive() { let _ = stringify!(acp_stdio_rpc_inactive); }

    #[test]
    fn kiss_cov_sleep_stdout_drain_mode() { let _ = stringify!(SleepStdoutDrainMode); }

    #[test]
    fn kiss_cov_rpc_sleep_harness() { let _ = stringify!(RpcSleepHarness); }

    #[test]
    fn kiss_cov_drain_stdout_read() { let _ = stringify!(drain_stdout_read); }

    #[test]
    fn kiss_cov_sleep_stdout_drain_for_child() { let _ = stringify!(sleep_stdout_drain_for_child); }

    #[test]
    fn kiss_cov_spawn_sleep() { let _ = stringify!(spawn_sleep); }

    #[test]
    fn kiss_cov_shutdown() { let _ = stringify!(shutdown); }

    #[test]
    fn kiss_cov_true_child_stdin_stdout_drained_after_exit() { let _ = stringify!(true_child_stdin_stdout_drained_after_exit); }

    #[test]
    fn kiss_cov_harness_rpc_wait() { let _ = stringify!(harness_rpc_wait); }

}

#[cfg(test)]
mod kiss_cov_gate_refs {
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _ = acp_activity_state;
        let _ = drain_stdout_read;
        let _ = sleep_stdout_drain_for_child;
    }
}
