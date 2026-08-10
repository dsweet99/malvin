use std::collections::HashSet;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use super::{watch_process_group_memory_with_rss_sampler, MemWatchHandles};
use crate::artifacts::create_kpop_run_artifacts;
use crate::sandbox_oom::{gate_iteration_oom_killed, OOM_REASON_MEMORY_LIMIT};

#[cfg(unix)]
fn spawn_sleep_child_in_new_process_group() -> (tokio::process::Child, u32, HashSet<u32>) {
    crate::test_utils::enable_test_fast_teardown();
    let baseline = crate::acp::snapshot_pids();
    let mut cmd = tokio::process::Command::new("sleep");
    unsafe {
        cmd.arg("30").pre_exec(|| {
            // Put the child in its own process group (pgid == pid).
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

/// Regression: when RSS/PSS measurement returns `None`, the watcher must fail-closed
/// (terminate after brief consecutive sample failures), not treat unknown as under limit.
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

/// OOM teardown must persist a malvin-owned marker for gate retry attribution.
#[cfg(unix)]
#[tokio::test]
async fn watch_process_group_memory_writes_sandbox_oom_marker() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = create_kpop_run_artifacts("code", Some(tmp.path())).expect("artifacts");
    let (_child, pgid, baseline) = spawn_sleep_child_in_new_process_group();
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

#[cfg(test)]
mod kiss_cov_auto {
    #[cfg(unix)]
    use super::*;

    #[cfg(unix)]
    #[test]
    fn kiss_cov_watch_process_group_memory_fail_closed_when_rss_unavailable() {
        let _ = watch_process_group_memory_fail_closed_when_rss_unavailable;
    }
    #[cfg(unix)]
    #[test]
    fn kiss_cov_watch_process_group_memory_writes_sandbox_oom_marker() {
        let _ = watch_process_group_memory_writes_sandbox_oom_marker;
    }

    #[test]
    fn kiss_cov_watch_process_group_memory_with_rss_sampler() {
        let _ = super::super::watch_process_group_memory_with_rss_sampler;
    }
}
