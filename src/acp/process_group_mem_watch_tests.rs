use std::collections::HashSet;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use super::{
    record_sandbox_oom_marker, watch_process_group_memory_with_rss_sampler, MemWatchHandles,
};
use crate::artifacts::create_kpop_run_artifacts;
use crate::sandbox_oom::{
    gate_iteration_oom_killed, SandboxOomKillFacts, OOM_REASON_MEASUREMENT_FAIL_CLOSED,
    OOM_REASON_MEMORY_LIMIT,
};

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

/// Fail-closed when USS samples are `None`.
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

/// `reader_dead`: no fail-closed on `None` USS samples.
#[cfg(unix)]
#[tokio::test]
async fn watch_process_group_memory_no_fail_closed_when_reader_dead() {
    let (mut child, pgid, baseline) = spawn_sleep_child_in_new_process_group();
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
    let _ = child.kill().await;
    let _ = child.wait().await;
}

/// `reader_dead`: still kill on hard over-limit.
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

/// Persist OOM marker for gate retry attribution.
#[cfg(unix)]
#[tokio::test]
async fn watch_process_group_memory_writes_sandbox_oom_marker() {
    let _guard = crate::test_utils::test_env_lock();
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

/// Non-kpop runs still write `sandbox_oom.json` (`gate_iteration` 0).
#[cfg(unix)]
#[tokio::test]
async fn watch_process_group_memory_writes_marker_without_gate_iteration() {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = create_kpop_run_artifacts("code", Some(tmp.path())).expect("artifacts");
    let (_child, pgid, baseline) = spawn_sleep_child_in_new_process_group();
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

#[cfg(unix)]
#[test]
fn record_sandbox_oom_marker_noops_without_run_dir() {
    record_sandbox_oom_marker(
        None,
        SandboxOomKillFacts {
            reason: OOM_REASON_MEASUREMENT_FAIL_CLOSED,
            rss_bytes: None,
            limit_bytes: 1,
            pgid: 1,
        },
    );
}

#[cfg(unix)]
#[test]
fn record_sandbox_oom_marker_writes_when_gate_iteration_unset() {
    let _lock = crate::test_utils::test_env_lock();
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = create_kpop_run_artifacts("code", Some(tmp.path())).expect("artifacts");
    crate::gate_loop_session::set_active_gate_iteration(None);
    record_sandbox_oom_marker(
        Some(&artifacts.run_dir),
        SandboxOomKillFacts {
            reason: OOM_REASON_MEASUREMENT_FAIL_CLOSED,
            rss_bytes: None,
            limit_bytes: 1,
            pgid: 1,
        },
    );
    assert!(gate_iteration_oom_killed(&artifacts, 0));
    let text = std::fs::read_to_string(artifacts.sandbox_oom_json_path()).expect("read");
    assert!(text.contains(OOM_REASON_MEASUREMENT_FAIL_CLOSED));
}

#[cfg(unix)]
#[test]
fn record_sandbox_oom_marker_writes_fail_closed_reason() {
    let _lock = crate::test_utils::test_env_lock();
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = create_kpop_run_artifacts("code", Some(tmp.path())).expect("artifacts");
    crate::gate_loop_session::set_active_gate_iteration(Some(1));
    record_sandbox_oom_marker(
        Some(&artifacts.run_dir),
        SandboxOomKillFacts {
            reason: OOM_REASON_MEASUREMENT_FAIL_CLOSED,
            rss_bytes: None,
            limit_bytes: 512,
            pgid: 7,
        },
    );
    crate::gate_loop_session::set_active_gate_iteration(None);
    assert!(gate_iteration_oom_killed(&artifacts, 1));
    let text = std::fs::read_to_string(artifacts.sandbox_oom_json_path()).expect("read");
    assert!(text.contains(OOM_REASON_MEASUREMENT_FAIL_CLOSED));
}

#[cfg(test)]
mod kiss_cov_auto {
    #[cfg(unix)]
    use super::*;

    #[cfg(unix)]
    #[test]
    fn kiss_cov_watch_process_group_memory_cases() {
        let _ = watch_process_group_memory_fail_closed_when_rss_unavailable;
        let _ = watch_process_group_memory_no_fail_closed_when_reader_dead;
        let _ = watch_process_group_memory_still_kills_over_limit_when_reader_dead;
        let _ = watch_process_group_memory_writes_sandbox_oom_marker;
        let _ = watch_process_group_memory_writes_marker_without_gate_iteration;
        let _ = record_sandbox_oom_marker_writes_when_gate_iteration_unset;
        let _ = record_sandbox_oom_marker_writes_fail_closed_reason;
        let _ = super::super::watch_process_group_memory_with_rss_sampler;
    }
}
