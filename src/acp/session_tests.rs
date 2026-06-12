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
fn prompt_stdout_replacement_is_always_none() {
    assert_eq!(crate::acp::prompt_stdout_replacement(crate::output::MALVIN_WHO), None);
    assert_eq!(crate::acp::prompt_stdout_replacement("learn"), None);
}

pub(crate) fn mem_watch_test_spawn_args(cwd: &Path) -> AcpSpawnArgs<'_> {
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

pub(crate) fn mem_watch_test_telemetry() -> SessionReaderTelemetry {
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

pub(crate) fn spawn_sleep_child_in_new_process_group(
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

pub(crate) fn acp_session_from_sleep_child(
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
            spawn_pid_baseline: super::unix_process_group_ps::snapshot_pids(),
            session_id: "mem-watch-test".into(),
            rpc_timeout: args.rpc_timeout,
            telemetry: mem_watch_test_telemetry(),
            work_dir: cwd.to_path_buf(),
            run_timing: None,
        },
    )))
}

pub(crate) fn session_with_sleep_child_for_mem_watch(cwd: &Path) -> (AcpSession, u32) {
    crate::test_utils::enable_test_fast_teardown();
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
        spawn_pid_baseline: session.0.spawn_pid_baseline.clone(),
        run_dir: None,
    })
    .await;
    let status = session
        .0
        .child
        .lock()
        .await
        .as_mut()
        .expect("child")
        .wait()
        .await
        .expect("wait");
    assert!(!status.success());
}

#[cfg(unix)]
fn spawn_sleep_seconds(seconds: &str, isolated_pg: bool) -> (std::process::Child, u32) {
    use std::os::unix::process::CommandExt;
    use std::process::Command;
    let mut cmd = Command::new("sleep");
    cmd.arg(seconds);
    if isolated_pg {
        cmd.process_group(0);
    }
    let child = cmd.spawn().expect("spawn sleep");
    let pid = child.id();
    (child, pid)
}

/// Malvin-spawned children outside the agent PG (e.g. repo gates) must count toward sandbox RSS.
#[cfg(unix)]
#[test]
fn malvin_child_outside_agent_pg_counts_in_sandbox_rss() {
    let baseline = super::unix_process_group_ps::snapshot_pids();
    let (mut agent_child, agent_pgid) = spawn_sleep_seconds("120", true);
    let (mut gate_child, gate_pid) = spawn_sleep_seconds("120", false);
    std::thread::sleep(std::time::Duration::from_millis(100));
    let rss = crate::malvin_sandbox::malvin_session_rss_bytes(Some(agent_pgid), &baseline)
        .expect("sandbox rss");
    let monitor = crate::acp::sandbox_monitor_pids(Some(agent_pgid), &baseline);
    assert!(monitor.contains(&agent_pgid));
    assert!(monitor.contains(&gate_pid));
    assert!(rss > 0);
    let _ = agent_child.kill();
    let _ = gate_child.kill();
    let _ = agent_child.wait();
    let _ = gate_child.wait();
}

#[tokio::test]
async fn spawn_process_group_memory_watcher_starts_for_session() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (session, _pgid) = session_with_sleep_child_for_mem_watch(tmp.path());
    spawn_process_group_memory_watcher(&session, tmp.path());
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(session.is_alive().await);
}

/// Regression: when the agent process group is already empty, the mem watcher must still
/// tear down reparented `setsid` orphans (not return early on `!process_group_still_alive`).
#[cfg(unix)]
#[tokio::test]
async fn watch_process_group_memory_kills_orphan_after_agent_pg_exits() {
    use std::sync::atomic::AtomicBool;

    use super::hostile_orphan_test_util::{
        process_alive, read_orphan_pid, spawn_hostile_agent_exits_after_orphan_fork,
    };

    crate::test_utils::enable_test_fast_teardown();
    let tmp = tempfile::tempdir().expect("tempdir");
    let orphan_pid_file = tmp.path().join("orphan.pid");
    let spawn_baseline = super::unix_process_group_ps::snapshot_pids();
    let (mut agent, pgid) =
        spawn_hostile_agent_exits_after_orphan_fork(tmp.path(), &orphan_pid_file);
    let orphan_pid = read_orphan_pid(&orphan_pid_file).await;
    assert!(
        process_alive(orphan_pid),
        "setup: setsid orphan should be running"
    );
    let agent_status = agent.wait().expect("wait agent");
    assert!(agent_status.success() || agent_status.code() == Some(0));
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert!(
        !process_alive(pgid),
        "setup: agent process group leader should have exited"
    );
    watch_process_group_memory(crate::acp::process_group_mem_watch::MemWatchHandles {
        reader_dead: Arc::new(AtomicBool::new(false)),
        pgid,
        limit_bytes: 1,
        spawn_pid_baseline: spawn_baseline,
        run_dir: None,
    })
    .await;
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert!(
        !process_alive(orphan_pid),
        "mem watcher must kill setsid orphans after agent PG is gone (orphan_pid={orphan_pid})"
    );
}

/// Regression: OOM mem watcher must pass `spawn_pid_baseline` into agent teardown so
/// reparented `setsid` session-leader orphans die (not only PG members).
#[cfg(unix)]
#[tokio::test]
async fn watch_process_group_memory_kills_setsid_orphan_on_oom() {
    use std::sync::atomic::AtomicBool;

    use super::hostile_orphan_test_util::{process_alive, read_orphan_pid, spawn_hostile_agent};

    crate::test_utils::enable_test_fast_teardown();
    let tmp = tempfile::tempdir().expect("tempdir");
    let orphan_pid_file = tmp.path().join("orphan.pid");
    let spawn_baseline = super::unix_process_group_ps::snapshot_pids();
    let (mut agent, pgid) = spawn_hostile_agent(tmp.path(), &orphan_pid_file);
    let orphan_pid = read_orphan_pid(&orphan_pid_file).await;
    assert!(
        process_alive(orphan_pid),
        "setup: setsid orphan should be running before OOM teardown"
    );
    watch_process_group_memory(crate::acp::process_group_mem_watch::MemWatchHandles {
        reader_dead: Arc::new(AtomicBool::new(false)),
        pgid,
        limit_bytes: 1,
        spawn_pid_baseline: spawn_baseline,
        run_dir: None,
    })
    .await;
    let _ = agent.wait();
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert!(
        !process_alive(orphan_pid),
        "OOM mem watcher must kill reparented setsid orphans (orphan_pid={orphan_pid})"
    );
}

#[cfg(all(test, unix))] #[test] fn kiss_cov_session_mem_watch_test_names() { let _ = (watch_process_group_memory_kills_over_limit_child, spawn_process_group_memory_watcher_starts_for_session, watch_process_group_memory_kills_orphan_after_agent_pg_exits, watch_process_group_memory_kills_setsid_orphan_on_oom); }
