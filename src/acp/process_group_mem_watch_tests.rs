use std::sync::Arc;

use super::{watch_process_group_memory_with_rss_sampler, MemWatchHandles};
use crate::acp::session_tests::session_with_sleep_child_for_mem_watch;

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
