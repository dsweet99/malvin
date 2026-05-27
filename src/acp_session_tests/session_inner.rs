use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

pub fn dead_transport_child_stdio() -> (
    tokio::sync::Mutex<Option<tokio::process::Child>>,
    Arc<tokio::sync::Mutex<tokio::process::ChildStdin>>,
) {
    let mut child = tokio::process::Command::new("cat")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("spawn cat");
    let stdin = Arc::new(tokio::sync::Mutex::new(child.stdin.take().expect("stdin")));
    (tokio::sync::Mutex::new(Some(child)), stdin)
}

pub fn dead_transport_sync_channels() -> (
    Arc<std::sync::atomic::AtomicU64>,
    Arc<tokio::sync::Notify>,
    Arc<std::sync::atomic::AtomicBool>,
    Arc<std::sync::atomic::AtomicU64>,
) {
    (
        Arc::new(std::sync::atomic::AtomicU64::new(0)),
        Arc::new(tokio::sync::Notify::new()),
        Arc::new(std::sync::atomic::AtomicBool::new(true)),
        Arc::new(std::sync::atomic::AtomicU64::new(1)),
    )
}

pub fn dead_transport_session_inner() -> crate::acp::AcpSessionInner {
    let (child, stdin) = dead_transport_child_stdio();
    let (acp_activity_seq, acp_activity_notify, reader_dead, next_id) =
        dead_transport_sync_channels();
    crate::acp::AcpSessionInner {
        child,
        process_group_id: None,
        spawn_pid_baseline: std::collections::HashSet::new(),
        stdin,
        pending: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::default())),
        acp_activity_seq,
        acp_activity_notify,
        next_id,
        session_id: "session-id".to_string(),
        reader_dead,
        rpc_timeout: Duration::from_millis(100),
        busy: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        trace_writer: Arc::new(tokio::sync::Mutex::new(None)),
        prompt_rpc_id: Arc::new(std::sync::atomic::AtomicU64::new(123)),
        prompt_singleflight: Arc::new(tokio::sync::Mutex::new(())),
        acp_verbose: false,
        ui_idle_notify: None,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        prompts_log_run_dir: None,
        log_full_outgoing_prompts: false,
        trace_jsonl: None,
        prompt_round_health: Arc::new(Mutex::new(crate::acp::PromptRoundHealth::default())),
        work_dir: std::env::temp_dir(),
        run_timing: None,
    }
}

#[cfg(test)]
mod session_inner_inline_tests {
    use super::{
        dead_transport_child_stdio, dead_transport_session_inner, dead_transport_sync_channels,
    };

    #[tokio::test]
    async fn dead_transport_session_inner_helpers() {
        let _ = dead_transport_child_stdio();
        let _ = dead_transport_sync_channels();
        let _ = dead_transport_session_inner();
    }
}
