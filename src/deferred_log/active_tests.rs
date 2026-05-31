use super::{register, try_push, unregister, SharedDeferSink, try_log};
use crate::deferred_log::{
    build_display_log_entry, test_fixtures::zero_age_defer_shared, DeferredLogSink,
};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

fn defer_heartbeat_under_held_mutex(shared: &SharedDeferSink) {
    let (display, log_line) = crate::output::stdout_heartbeat_display_and_log_line(
        crate::output::WHO_H,
        "HB: 20260524.000000",
        Some("20260524.000000.000"),
    );
    let _acp_hold = shared.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    assert!(crate::output::try_defer_heartbeat(&display, &log_line));
    assert!(!display.starts_with("20"));
}

fn defer_heartbeat_split_log() -> String {
    let tmp = tempfile::tempdir().expect("tempdir");
    let log_path = tmp.path().join("stdout.log");
    crate::output::set_stdout_log_path(Some(log_path.clone()));
    let shared = zero_age_defer_shared("hb_defer_split");
    register(Arc::clone(&shared));
    defer_heartbeat_under_held_mutex(&shared);
    unregister();
    shared
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .force_flush();
    crate::output::set_stdout_log_path(None);
    std::fs::read_to_string(log_path).unwrap_or_default()
}

fn heartbeat_contention_pending_len() -> usize {
    let shared = zero_age_defer_shared("single_hb");
    register(Arc::clone(&shared));
    let pending = {
        let _acp_hold = shared.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        crate::output::test_set_last_heartbeat_elapsed(Duration::from_secs(61));
        let (display, log) = crate::output::stdout_heartbeat_display_and_log_line(
            crate::output::MALVIN_WHO,
            "HB: 20260524.000000",
            Some("20260524.000000.000"),
        );
        crate::output::write_heartbeat_log_line(&display, &log);
        assert!(try_log(build_display_log_entry(
            "CONTENDED_TAG".into(),
            "CONTENDED_TAG".into(),
        )));
        super::pending_len()
    };
    unregister();
    pending
}

fn try_push_heartbeat_while_mutex_held(shared: &SharedDeferSink) {
    let hb = build_display_log_entry(
        "h| HB: 20260524.000000".into(),
        "20260524.000000.000 h|HB: 20260524.000000".into(),
    );
    let _acp_hold = shared
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    assert!(
        try_push(hb),
        "wall-clock defer hook must not drop heartbeat when ACP path holds the sink mutex"
    );
    assert_eq!(super::pending_len(), 1, "heartbeat must sit in pending until mutex released");
}

#[test]
fn try_push_queues_heartbeat_when_defer_sink_mutex_held() {
    let _stdout_guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    crate::output::reset_stdout_heartbeat_for_test();
    let shared: SharedDeferSink = Arc::new(std::sync::Mutex::new(DeferredLogSink::new(
        "contention".to_string(),
        PathBuf::new(),
        crate::deferred_log::config::DeferredLogConfig {
            max_age: std::time::Duration::from_secs(3600),
            max_drain_per_log: 64,
            cursor_dir: PathBuf::new(),
        },
    )));
    register(Arc::clone(&shared));
    try_push_heartbeat_while_mutex_held(&shared);
    shared
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push_entry(build_display_log_entry("flush".into(), "flush".into()));
    assert_eq!(super::pending_len(), 0);
    assert!(
        shared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .queue_len()
            >= 1,
        "heartbeat must reach defer sink queue after flush"
    );
    unregister();
}

#[test]
fn pending_flush_covers_queue_and_drain_paths() {
    let shared: SharedDeferSink = Arc::new(std::sync::Mutex::new(DeferredLogSink::new(
        "pending".to_string(),
        PathBuf::new(),
        crate::deferred_log::config::DeferredLogConfig::from_env(),
    )));
    super::queue_pending(build_display_log_entry(
        "malvin.| pending-hb".into(),
        "20260524.000000.000 malvin.|pending-hb".into(),
    ));
    assert_eq!(super::pending_len(), 1);
    super::flush_pending_into(
        &mut shared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
    );
    assert_eq!(super::pending_len(), 0);
    unregister();
}

#[test]
fn active_slot_register_unregister_roundtrip() {
    let shared: SharedDeferSink = Arc::new(std::sync::Mutex::new(DeferredLogSink::new(
        "active".to_string(),
        PathBuf::new(),
        crate::deferred_log::config::DeferredLogConfig::from_env(),
    )));
    register(Arc::clone(&shared));
    assert!(super::is_registered());
    assert!(try_log(build_display_log_entry("d".into(), "l".into())));
    assert!(try_log(build_display_log_entry("hb-d".into(), "hb-l".into())));
    unregister();
    assert!(!super::is_registered());
    assert!(!try_log(build_display_log_entry("d".into(), "l".into())));
    register(Arc::clone(&shared));
    unregister();
}

#[test]
fn defer_heartbeat_under_held_mutex_defers_display_from_log() {
    let _stdout_guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    crate::deferred_log::install_stdout_hooks();
    let shared = zero_age_defer_shared("hb_direct");
    register(Arc::clone(&shared));
    defer_heartbeat_under_held_mutex(&shared);
    unregister();
}

#[test]
fn try_defer_heartbeat_under_mutex_flushes_display_log_split() {
    let _stdout_guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _heartbeat_guard = crate::output::HEARTBEAT_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    crate::output::reset_stdout_heartbeat_for_test();
    crate::deferred_log::install_stdout_hooks();
    let text = defer_heartbeat_split_log();
    let line = text
        .lines()
        .find(|l| l.contains("HB:"))
        .expect("heartbeat log line");
    assert!(line.contains("20260524.000000.000 h|HB:"));
}

#[test]
fn unregister_spills_orphaned_pending_without_active_sink() {
    let _stdout_guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    while super::is_registered() {
        unregister();
    }
    let (terminal, text) =
        crate::deferred_log::test_fixtures::capture_stdout_render_unlocked(|| {
            super::queue_pending(build_display_log_entry(
                "malvin.| ORPHAN_ACTIVE".into(),
                "20260524.000000.000 malvin.|ORPHAN_ACTIVE".into(),
            ));
            unregister();
        });
    assert!(text.contains("ORPHAN_ACTIVE"));
    assert!(terminal.contains("malvin.| ORPHAN_ACTIVE"));
    assert!(!terminal.starts_with("20"));
}

#[test]
fn try_log_under_mutex_queues_tagged_and_bundled_heartbeat() {
    let _stdout_guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _heartbeat_guard = crate::output::HEARTBEAT_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    crate::output::reset_stdout_heartbeat_for_test();
    crate::output::test_set_last_heartbeat_elapsed(Duration::from_secs(61));
    crate::deferred_log::install_stdout_hooks();
    assert_eq!(heartbeat_contention_pending_len(), 2);
}

#[test]
fn contention_flush_emits_one_heartbeat_to_terminal_and_log() {
    let _heartbeat_guard = crate::output::HEARTBEAT_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    crate::output::reset_stdout_heartbeat_for_test();
    crate::deferred_log::install_stdout_hooks();
    let (terminal, log) = crate::deferred_log::test_fixtures::capture_stdout_render(|| {
        let shared = zero_age_defer_shared("contention_flush");
        register(Arc::clone(&shared));
        crate::output::test_set_last_heartbeat_elapsed(Duration::from_secs(61));
        let (display, log_line) = crate::output::stdout_heartbeat_display_and_log_line(
            crate::output::MALVIN_WHO,
            "HB: 20260524.000000",
            Some("20260524.000000.000"),
        );
        {
            let _acp_hold = shared.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            crate::output::write_heartbeat_log_line(&display, &log_line);
            assert!(try_log(build_display_log_entry(
                "CONTENDED_TAG".into(),
                "CONTENDED_TAG".into(),
            )));
        }
        unregister();
        shared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .force_flush();
    });
    assert_eq!(terminal.lines().filter(|l| l.contains("HB:")).count(), 1);
    assert_eq!(log.lines().filter(|l| l.contains("HB:")).count(), 1);
    assert!(!terminal.starts_with("20"));
    assert!(log.lines().next().unwrap_or("").starts_with("20260524"));
}
