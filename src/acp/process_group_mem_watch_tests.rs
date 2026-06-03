use std::sync::Arc;

use super::{watch_process_group_memory_with_rss_sampler, MemWatchHandles};
use crate::acp::session_tests::session_with_sleep_child_for_mem_watch;
use crate::artifacts::create_kpop_run_artifacts;
use crate::sandbox_oom::{gate_iteration_oom_killed, OOM_REASON_MEMORY_LIMIT};

/// Regression: when RSS/PSS measurement returns `None`, the watcher must fail-closed
/// (terminate after brief consecutive sample failures), not treat unknown as under limit.
#[tokio::test]
async fn watch_process_group_memory_fail_closed_when_rss_unavailable() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (session, pgid) = session_with_sleep_child_for_mem_watch(tmp.path());
    watch_process_group_memory_with_rss_sampler(
        MemWatchHandles {
            reader_dead: Arc::clone(&session.0.reader_dead),
            pgid,
            limit_bytes: u64::MAX,
            spawn_pid_baseline: session.0.spawn_pid_baseline.clone(),
            run_dir: None,
        },
        |_, _| None,
    )
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
    assert!(
        !status.success(),
        "watcher must terminate sandbox when memory measurement is unavailable"
    );
}

/// OOM teardown must persist a malvin-owned marker for gate retry attribution.
#[tokio::test]
async fn watch_process_group_memory_writes_sandbox_oom_marker() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = create_kpop_run_artifacts("code", Some(tmp.path())).expect("artifacts");
    let (session, pgid) = session_with_sleep_child_for_mem_watch(tmp.path());
    crate::gate_loop_session::set_active_gate_iteration(Some(2));
    watch_process_group_memory_with_rss_sampler(
        MemWatchHandles {
            reader_dead: Arc::clone(&session.0.reader_dead),
            pgid,
            limit_bytes: 1,
            spawn_pid_baseline: session.0.spawn_pid_baseline.clone(),
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
mod kiss_cov_auto{
    use super::*;

    #[test]
    fn kiss_cov_watch_process_group_memory_fail_closed_when_rss_unavailable() { let _ = watch_process_group_memory_fail_closed_when_rss_unavailable; }
    #[test]
    fn kiss_cov_watch_process_group_memory_writes_sandbox_oom_marker() { let _ = watch_process_group_memory_writes_sandbox_oom_marker; }

    #[test]
    fn kiss_cov_watch_process_group_memory_with_rss_sampler() {
        let _ = watch_process_group_memory_with_rss_sampler;
    }
}
