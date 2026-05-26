use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use crate::acp::process_group_mem_watch::{
    spawn_process_group_memory_watcher, watch_process_group_memory,
};
use crate::acp::session_types::{SessionChannelState, SessionReaderTelemetry};
use crate::acp::{AcpSession, AcpSpawnArgs};
use tokio::process::Command;

#[test]
fn prompt_stdout_replacement_maps_learn_placeholder() {
    assert_eq!(crate::acp::prompt_stdout_replacement(crate::output::MALVIN_WHO), None);
    assert_eq!(
        crate::acp::prompt_stdout_replacement("learn"),
        Some(crate::malvin_constants::LEARNING_PLACEHOLDER)
    );
}

fn mem_watch_test_spawn_args(cwd: &Path) -> AcpSpawnArgs<'_> {
    AcpSpawnArgs {
        cwd,
        bin_override: None,
        api_key: Some("k"),
        auth_token: Some("t"),
        rpc_timeout: Duration::from_secs(5),
        acp_verbose: false,
        george_acp_lane: None,
        ui_idle_notify: None,
        model: None,
        force: false,
        tee_trace_stdout: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        prompts_log_run_dir: None,
        log_full_outgoing_prompts: false,
    }
}

fn mem_watch_test_telemetry() -> SessionReaderTelemetry {
    SessionReaderTelemetry {
        acp_verbose: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        prompts_log_run_dir: None,
        log_full_outgoing_prompts: false,
        trace_jsonl: None,
    }
}

fn spawn_sleep_child_in_new_process_group(
    cwd: &Path,
) -> (tokio::process::Child, tokio::process::ChildStdin, u32) {
    let mut cmd = Command::new("sleep");
    cmd.arg("120");
    cmd.current_dir(cwd);
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.as_std_mut().process_group(0);
    }
    let mut child = cmd.spawn().expect("spawn sleep");
    let pgid = child.id().expect("pgid");
    let stdin = child.stdin.take().expect("stdin");
    let _ = child.stdout.take();
    (child, stdin, pgid)
}

fn acp_session_from_sleep_child(
    cwd: &Path,
    child: tokio::process::Child,
    stdin: tokio::process::ChildStdin,
    pgid: u32,
) -> AcpSession {
    let args = mem_watch_test_spawn_args(cwd);
    let ch = SessionChannelState::new(stdin, &args);
    AcpSession(Arc::new(ch.into_session_inner(
        crate::acp::session_types::SessionInnerAssembly {
            child,
            process_group_id: Some(pgid),
            session_id: "mem-watch-test".into(),
            rpc_timeout: args.rpc_timeout,
            telemetry: mem_watch_test_telemetry(),
            work_dir: cwd.to_path_buf(),
            run_timing: None,
        },
    )))
}

fn session_with_sleep_child_for_mem_watch(cwd: &Path) -> (AcpSession, u32) {
    let (child, stdin, pgid) = spawn_sleep_child_in_new_process_group(cwd);
    let session = acp_session_from_sleep_child(cwd, child, stdin, pgid);
    (session, pgid)
}

#[tokio::test]
async fn watch_process_group_memory_kills_over_limit_child() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (session, pgid) = session_with_sleep_child_for_mem_watch(tmp.path());
    watch_process_group_memory(crate::acp::process_group_mem_watch::MemWatchHandles {
        reader_dead: Arc::clone(&session.0.reader_dead),
        pgid,
        limit_bytes: 1,
    })
    .await;
    let status = session
        .0
        .child
        .lock()
        .await
        .wait()
        .await
        .expect("wait");
    assert!(!status.success());
}

#[tokio::test]
async fn spawn_process_group_memory_watcher_starts_for_session() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (session, _pgid) = session_with_sleep_child_for_mem_watch(tmp.path());
    spawn_process_group_memory_watcher(&session, tmp.path());
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(session.is_alive().await);
}
