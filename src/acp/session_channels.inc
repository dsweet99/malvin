// Channel state built before the ACP JSON-RPC handshake completes.
use rand::{Rng, distributions::Alphanumeric};

/// Verbose logging for ACP (bundled for [`SessionChannelState::into_session_inner`]).
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone)]
pub struct SessionReaderTelemetry {
    pub acp_verbose: bool,
    /// When true, print raw output without timestamps/prefixes.
    pub raw_output: bool,
    /// When true, raw/plain stdout includes thought chunks.
    pub show_thoughts_on_stdout: bool,
    /// When true, allow styled markdown on stdout for tagged trace lines (`malvin code` / `malvin kpop`).
    pub emit_stdout_markdown: bool,
    pub prompts_log_run_dir: Option<PathBuf>,
    pub log_full_outgoing_prompts: bool,
    pub trace_jsonl: Option<Arc<AcpJsonlTrace>>,
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
    pub(crate) trace_jsonl: Option<Arc<AcpJsonlTrace>>,
}

fn acp_activity_state() -> (Arc<AtomicU64>, Arc<Notify>) {
    (Arc::new(AtomicU64::new(0)), Arc::new(Notify::new()))
}

fn random_agent_name() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(5)
        .map(char::from)
        .collect()
}

fn trace_jsonl_for_args(args: &AcpSpawnArgs<'_>) -> Option<Arc<AcpJsonlTrace>> {
    args.prompts_log_run_dir.map(|run_dir| {
        Arc::new(AcpJsonlTrace::new(
            run_dir.join("trace.jsonl"),
            random_agent_name(),
        ))
    })
}

pub struct SessionInnerAssembly {
    pub child: Child,
    pub process_group_id: Option<u32>,
    pub session_id: String,
    pub rpc_timeout: std::time::Duration,
    pub telemetry: SessionReaderTelemetry,
    pub work_dir: PathBuf,
    pub run_timing: Option<std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
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
            trace_jsonl: trace_jsonl_for_args(args),
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
            trace_jsonl: self.trace_jsonl.clone(),
        }
    }

    pub(crate) fn into_session_inner(self, parts: SessionInnerAssembly) -> AcpSessionInner {
        AcpSessionInner {
            child: Mutex::new(parts.child),
            process_group_id: parts.process_group_id,
            stdin: self.stdin,
            pending: self.pending,
            acp_activity_seq: self.acp_activity_seq,
            acp_activity_notify: self.acp_activity_notify,
            next_id: self.next_id,
            session_id: parts.session_id,
            reader_dead: self.reader_dead,
            rpc_timeout: parts.rpc_timeout,
            busy: self.busy,
            trace_writer: self.trace_writer,
            prompt_rpc_id: self.prompt_rpc_id,
            prompt_singleflight: self.prompt_singleflight,
            acp_verbose: parts.telemetry.acp_verbose,
            ui_idle_notify: self.ui_idle_notify,
            raw_output: parts.telemetry.raw_output,
            show_thoughts_on_stdout: parts.telemetry.show_thoughts_on_stdout,
            emit_stdout_markdown: parts.telemetry.emit_stdout_markdown,
            prompts_log_run_dir: parts.telemetry.prompts_log_run_dir,
            log_full_outgoing_prompts: parts.telemetry.log_full_outgoing_prompts,
            trace_jsonl: parts.telemetry.trace_jsonl,
            work_dir: parts.work_dir,
            run_timing: parts.run_timing,
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

#[cfg(test)]
async fn stdin_from_sleep_holder() -> (tokio::process::Child, ChildStdin) {
    let mut child = tokio::process::Command::new("sleep")
        .arg("60")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect("spawn sleep");
    let stdin = child.stdin.take().expect("stdin");
    (child, stdin)
}

#[tokio::test]
async fn session_channel_state_sets_trace_jsonl_when_prompts_log_run_dir_set() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let run_dir = tmp.path().join("_malvin").join("r");
    std::fs::create_dir_all(&run_dir).expect("run dir");
    let (mut child, stdin) = stdin_from_sleep_holder().await;
    let args = AcpSpawnArgs {
        cwd: tmp.path(),
        bin_override: None,
        api_key: None,
        auth_token: None,
        rpc_timeout: std::time::Duration::from_secs(1),
        acp_verbose: false,
        george_acp_lane: None,
        ui_idle_notify: None,
        model: None,
        force: false,
        sandbox: false,
        tee_trace_stdout: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        prompts_log_run_dir: Some(run_dir.as_path()),
        log_full_outgoing_prompts: false,
    };
    let state = SessionChannelState::new(stdin, &args);
    let trace = state.trace_jsonl.as_ref().expect("trace jsonl");
    assert_eq!(trace.path, run_dir.join("trace.jsonl"));
    assert_eq!(trace.name.len(), 5);
    assert!(trace.name.chars().all(|c| c.is_ascii_alphanumeric()));
    let _ = child.kill().await;
}

#[test]
fn acp_activity_state_returns_valid_arc_pair() {
    use std::sync::atomic::Ordering;
    let (seq, notify) = acp_activity_state();
    assert_eq!(seq.load(Ordering::SeqCst), 0);
    notify.notify_one();
}

