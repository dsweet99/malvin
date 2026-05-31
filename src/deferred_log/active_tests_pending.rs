use super::{register, try_push, unregister, SharedDeferSink, try_log};
use crate::deferred_log::{
    build_display_log_entry,
    test_fixtures::{aged_defer_shared, zero_age_defer_shared},
};
use std::sync::Arc;

fn flush_unregister(shared: &SharedDeferSink) {
    unregister();
    shared
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .force_flush();
}

struct StdoutLogCtx {
    _stdout_guard: std::sync::MutexGuard<'static, ()>,
    _tmp: tempfile::TempDir,
    path: std::path::PathBuf,
}

impl StdoutLogCtx {
    fn new() -> Self {
        let stdout_guard = crate::output::STDOUT_LOG_TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("stdout.log");
        crate::output::set_stdout_log_path(Some(path.clone()));
        Self {
            _stdout_guard: stdout_guard,
            _tmp: tmp,
            path,
        }
    }

    fn finish(self) -> String {
        crate::output::set_stdout_log_path(None);
        std::fs::read_to_string(self.path).unwrap_or_default()
    }
}

fn assert_fifo_order(text: &str, first: &str, second: &str) {
    let pos_first = text.find(first).expect("first marker in stdout.log");
    let pos_second = text.find(second).expect("second marker in stdout.log");
    assert!(
        pos_first < pos_second,
        "plan FIFO: {first} before {second}; log={text:?}"
    );
}

#[test]
fn pending_entries_emitted_on_unregister_then_force_flush() {
    let text = {
        let log = StdoutLogCtx::new();
        let shared = zero_age_defer_shared("teardown");
        register(Arc::clone(&shared));
        let marker = "PENDING_TEARDOWN_MARKER";
        let hb = build_display_log_entry(
            format!("malvin.| {marker}"),
            format!("20260524.000000.000 malvin.| {marker}"),
        );
        {
            let _acp_hold = shared.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            assert!(try_push(hb));
            assert_eq!(super::pending_len(), 1);
        }
        flush_unregister(&shared);
        log.finish()
    };
    assert!(text.contains("PENDING_TEARDOWN_MARKER"));
}

#[test]
fn unregister_without_flush_clears_global_pending() {
    let _stdout_guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    crate::output::reset_stdout_heartbeat_for_test();
    crate::output::test_set_last_heartbeat_elapsed(std::time::Duration::from_secs(61));
    let shared = zero_age_defer_shared("pending_stale");
    register(Arc::clone(&shared));
    {
        let _acp_hold = shared.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(try_log(build_display_log_entry("STALE_TAG".into(), "STALE_TAG".into())));
        assert_eq!(super::pending_len(), 2);
    }
    unregister();
    assert_eq!(super::pending_len(), 0);
}

#[test]
fn unregister_emits_orphaned_pending_without_active_sink() {
    let text = {
        let log = StdoutLogCtx::new();
        super::queue_pending(build_display_log_entry(
            "ORPHAN_PENDING".into(),
            "ORPHAN_PENDING".into(),
        ));
        unregister();
        log.finish()
    };
    assert!(text.contains("ORPHAN_PENDING"));
}

fn fifo_spill_unregister_text() -> String {
    let log = StdoutLogCtx::new();
    let shared = aged_defer_shared("fifo_spill");
    register(Arc::clone(&shared));
    shared
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push_entry(build_display_log_entry(
            "QUEUED_FIRST".into(),
            "QUEUED_FIRST".into(),
        ));
    {
        let _acp_hold = shared.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(try_log(build_display_log_entry(
            "PENDING_SECOND".into(),
            "PENDING_SECOND".into(),
        )));
    }
    unregister();
    shared
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .force_flush();
    log.finish()
}

#[test]
fn unregister_spill_preserves_fifo_with_queued_sink_entry() {
    crate::output::reset_stdout_heartbeat_for_test();
    let text = fifo_spill_unregister_text();
    assert_fifo_order(&text, "QUEUED_FIRST", "PENDING_SECOND");
}

#[test]
fn unregister_while_sink_locked_emits_pending_not_drops() {
    crate::output::reset_stdout_heartbeat_for_test();
    crate::output::test_set_last_heartbeat_elapsed(std::time::Duration::from_secs(61));
    let text = {
        let log = StdoutLogCtx::new();
        let shared = zero_age_defer_shared("lock_unregister");
        register(Arc::clone(&shared));
        let marker = "DROPPED_ON_UNREGISTER_LOCK";
        {
            let _acp_hold = shared.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            assert!(try_log(build_display_log_entry(marker.into(), marker.into())));
            unregister();
            assert_eq!(super::pending_len(), 0);
        }
        shared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .force_flush();
        log.finish()
    };
    assert!(text.contains("DROPPED_ON_UNREGISTER_LOCK"));
}

#[test]
fn try_log_pending_omits_heartbeat_when_not_due() {
    let _stdout_guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    crate::output::reset_stdout_heartbeat_for_test();
    let shared = zero_age_defer_shared("no_hb");
    register(Arc::clone(&shared));
    {
        let _acp_hold = shared.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(try_log(build_display_log_entry("TAG_ONLY".into(), "TAG_ONLY".into())));
        assert_eq!(super::pending_len(), 1);
    }
    unregister();
    assert_eq!(super::pending_len(), 0);
}

#[test]
fn try_log_pending_bundles_heartbeat_when_sink_mutex_held() {
    let _stdout_guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    crate::output::reset_stdout_heartbeat_for_test();
    crate::output::test_set_last_heartbeat_elapsed(std::time::Duration::from_secs(61));
    let shared = zero_age_defer_shared("try_log_hb");
    register(Arc::clone(&shared));
    {
        let _acp_hold = shared.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(try_log(build_display_log_entry(
            "CONTENDED_TAG".into(),
            "CONTENDED_TAG".into(),
        )));
        assert_eq!(super::pending_len(), 2);
    }
    unregister();
}
