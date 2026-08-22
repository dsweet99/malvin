use super::{
    MemWatchHandles, watch_process_group_memory_with_rss_sampler,
};
use crate::artifacts::{RunArtifacts, create_run_artifacts_from_text};
use crate::sandbox_oom::{
    OOM_REASON_MEMORY_LIMIT,
    gate_iteration_oom_killed,
};
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
#[cfg(unix)]
#[path = "process_group_mem_watch_test_support.rs"]
mod process_group_mem_watch_test_support;
#[cfg(unix)]
use process_group_mem_watch_test_support::spawn_std_sleep_child_in_new_process_group;
#[cfg(unix)]
fn spawn_sleep_child_in_new_process_group() -> (tokio::process::Child, u32, HashSet<u32>) {
    crate::test_utils::enable_test_fast_teardown();
    let baseline = crate::acp::snapshot_pids();
    let mut cmd = tokio::process::Command::new("sleep");
    unsafe {
        cmd.arg("30").pre_exec(|| {
            if libc::setpgid(0, 0) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let child = cmd.spawn().expect("spawn sleep");
    let pgid = child.id().expect("pid");
    (child, pgid, baseline)
}
#[cfg(unix)]
fn prepare_oom_marker_test() -> (
    tempfile::TempDir,
    RunArtifacts,
    tokio::process::Child,
    u32,
    HashSet<u32>,
) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = create_run_artifacts_from_text("code", Some(tmp.path())).expect("artifacts");
    let (child, pgid, baseline) = spawn_sleep_child_in_new_process_group();
    (tmp, artifacts, child, pgid, baseline)
}
#[cfg(unix)]
#[tokio::test]
async fn watch_process_group_memory_fail_closed_when_rss_unavailable() {
    let (mut child, pgid, baseline) = spawn_sleep_child_in_new_process_group();
    let reader_dead = Arc::new(AtomicBool::new(false));
    watch_process_group_memory_with_rss_sampler(
        MemWatchHandles {
            reader_dead: Arc::clone(&reader_dead),
            pgid,
            limit_bytes: u64::MAX,
            spawn_pid_baseline: baseline,
            run_dir: None,
        },
        |_, _| None,
    )
    .await;
    let status = child.wait().await.expect("wait");
    assert!(
        !status.success(),
        "watcher must terminate sandbox when memory measurement is unavailable"
    );
}
#[cfg(unix)]
#[tokio::test]
async fn watch_process_group_memory_no_fail_closed_when_reader_dead() {
    let (mut child, pgid, baseline) = spawn_std_sleep_child_in_new_process_group();
    let reader_dead = Arc::new(AtomicBool::new(true));
    let watch = watch_process_group_memory_with_rss_sampler(
        MemWatchHandles {
            reader_dead: Arc::clone(&reader_dead),
            pgid,
            limit_bytes: u64::MAX,
            spawn_pid_baseline: baseline,
            run_dir: None,
        },
        |_, _| None,
    );
    let raced = tokio::time::timeout(std::time::Duration::from_millis(80), watch).await;
    assert!(
        raced.is_err(),
        "with reader_dead, None samples must not terminate (fail-closed suppressed)"
    );
    assert!(
        child.try_wait().expect("try_wait").is_none(),
        "reader_dead + None USS must not kill the sandbox"
    );
    child.kill().expect("kill sleep");
    let _ = child.wait().expect("reap sleep");
}
#[cfg(unix)]
#[tokio::test]
async fn watch_process_group_memory_still_kills_over_limit_when_reader_dead() {
    let (mut child, pgid, baseline) = spawn_sleep_child_in_new_process_group();
    let reader_dead = Arc::new(AtomicBool::new(true));
    watch_process_group_memory_with_rss_sampler(
        MemWatchHandles {
            reader_dead: Arc::clone(&reader_dead),
            pgid,
            limit_bytes: 1,
            spawn_pid_baseline: baseline,
            run_dir: None,
        },
        |_, _| Some(999),
    )
    .await;
    let status = child.wait().await.expect("wait");
    assert!(
        !status.success(),
        "reader_dead must not disable hard over-limit enforcement"
    );
}
#[cfg(unix)]
#[tokio::test]
async fn watch_process_group_memory_writes_sandbox_oom_marker() {
    let _guard = crate::test_utils::test_env_lock();
    let _saved_env = crate::test_utils::SavedEnvVars::capture(&[
        "HOME",
        crate::MALVIN_TEST_ALLOW_HOME_CONFIG_MUTATION,
    ]);
    let home = tempfile::tempdir().expect("home");
    crate::test_utils::set_test_home_env(home.path());
    let (_tmp, artifacts, _child, pgid, baseline) = prepare_oom_marker_test();
    crate::gate_loop_session::set_active_gate_iteration(Some(2));
    watch_process_group_memory_with_rss_sampler(
        MemWatchHandles {
            reader_dead: Arc::new(AtomicBool::new(false)),
            pgid,
            limit_bytes: 1,
            spawn_pid_baseline: baseline,
            run_dir: Some(artifacts.run_dir.clone()),
        },
        |_, _| Some(999),
    )
    .await;
    crate::gate_loop_session::set_active_gate_iteration(None);
    assert!(gate_iteration_oom_killed(&artifacts, 2));
    let text = std::fs::read_to_string(artifacts.sandbox_oom_json_path()).expect("read");
    assert!(text.contains(OOM_REASON_MEMORY_LIMIT));
}
#[cfg(unix)]
#[tokio::test]
async fn watch_process_group_memory_writes_marker_without_gate_iteration() {
    let _guard = crate::test_utils::test_env_lock();
    let _saved_env = crate::test_utils::SavedEnvVars::capture(&[
        "HOME",
        crate::MALVIN_TEST_ALLOW_HOME_CONFIG_MUTATION,
    ]);
    let home = tempfile::tempdir().expect("home");
    crate::test_utils::set_test_home_env(home.path());
    let (_tmp, artifacts, _child, pgid, baseline) = prepare_oom_marker_test();
    crate::gate_loop_session::set_active_gate_iteration(None);
    watch_process_group_memory_with_rss_sampler(
        MemWatchHandles {
            reader_dead: Arc::new(AtomicBool::new(false)),
            pgid,
            limit_bytes: 1,
            spawn_pid_baseline: baseline,
            run_dir: Some(artifacts.run_dir.clone()),
        },
        |_, _| Some(999),
    )
    .await;
    assert!(gate_iteration_oom_killed(&artifacts, 0));
    let text = std::fs::read_to_string(artifacts.sandbox_oom_json_path()).expect("read");
    assert!(text.contains(OOM_REASON_MEMORY_LIMIT));
}
