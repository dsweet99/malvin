use super::*;
use crate::deferred_log::build_display_log_entry;

#[test]
fn pending_has_heartbeat_tracks_display_log_entries() {
    assert!(!pending_has_heartbeat());
    queue_pending(build_display_log_entry(
        "[malvin.........] HB: 20260524.000000".into(),
        "20260524.000000.000 [malvin.........] HB: 20260524.000000".into(),
    ));
    assert!(pending_has_heartbeat());
    assert!(entry_is_heartbeat(
        &build_display_log_entry("x".into(), "20260524.000000.000 [malvin.........] HB: 20260524.000000".into())
    ));
    pending_entries().clear();
    assert!(!pending_has_heartbeat());
}

#[test]
fn defer_already_has_heartbeat_sees_sink_queue() {
    let sink = crate::deferred_log::test_fixtures::aged_defer_shared("hb_seen");
    let empty = {
        let guard = sink.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        defer_already_has_heartbeat(&guard)
    };
    assert!(!empty);
    sink.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push_entry(build_display_log_entry(
            "[malvin.........] HB: 20260524.000000".into(),
            "20260524.000000.000 [malvin.........] HB: 20260524.000000".into(),
        ));
    let has = {
        let guard = sink.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        defer_already_has_heartbeat(&guard)
    };
    assert!(has);
}

#[test]
fn active_mutex_tracks_registration_state() {
    let registered = active_mutex().is_some();
    assert_eq!(registered, is_registered());
}

#[test]
fn spill_orphaned_pending_preserves_display_log_split() {
    let initial = pending_entries().len();
    assert_eq!(initial, pending_len());
    let (terminal, log) = crate::deferred_log::test_fixtures::capture_stdout_render(|| {
        queue_pending(build_display_log_entry(
            "[malvin.........] spill-inline".into(),
            "20260524.000000.000 [malvin.........] spill-inline".into(),
        ));
        spill_orphaned_pending();
    });
    assert!(terminal.contains("spill-inline"));
    assert!(log.contains("20260524.000000.000"));
    assert!(!terminal.starts_with("20"));
    let final_len = pending_entries().len();
    assert_eq!(final_len, pending_len());
}

#[test]
#[allow(unsafe_code)]
fn deferred_heartbeat_visible_after_unregister_without_force_flush() {
    use std::time::{Duration, Instant};

    let (display, log_line) = crate::output::stdout_heartbeat_display_and_log_line(
        crate::output::MALVIN_WHO,
        "HB: 20260524.000000",
        Some("20260524.000000.000"),
    );
    unsafe {
        std::env::set_var("MALVIN_DEFER_LOG_MAX_AGE_MS", "3600000");
    }
    let shared = crate::deferred_log::test_fixtures::aged_defer_shared("unregister_hb");
    crate::output::reset_stdout_heartbeat_for_test();
    crate::output::test_set_last_heartbeat_elapsed(Duration::from_secs(61));
    let (terminal, disk) = crate::deferred_log::test_fixtures::capture_stdout_render(|| {
        register(Arc::clone(&shared));
        crate::output::write_heartbeat_log_line(&display, &log_line);
        unregister();
    });
    unsafe {
        std::env::remove_var("MALVIN_DEFER_LOG_MAX_AGE_MS");
    }
    assert_eq!(
        shared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .queue_len(),
        0
    );
    assert_eq!(terminal.trim(), display);
    assert_eq!(disk.lines().next().expect("log line"), log_line);
    assert!(crate::output::heartbeat_rendered_if_due(Instant::now(), false).is_none());
}

#[test]
fn sync_sink_queue_heartbeat_flag_tracks_sink_queue() {
    let shared = crate::deferred_log::test_fixtures::aged_defer_shared("sync_flag");
    sync_sink_queue_heartbeat_flag(
        &shared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
    );
    shared
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push_entry(build_display_log_entry(
            "[malvin.........] HB: 20260524.000000".into(),
            "20260524.000000.000 [malvin.........] HB: 20260524.000000".into(),
        ));
    let cached = {
        let _hold = shared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        heartbeat_already_deferred(&shared)
    };
    assert!(cached);
}
