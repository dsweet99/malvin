use super::{register, try_log, try_push, unregister, SharedDeferSink};
use crate::deferred_log::{
    build_display_log_entry,
    test_fixtures::{aged_defer_shared, capture_stdout_render_unlocked, zero_age_defer_shared},
};
use std::sync::Arc;

fn heartbeat_entry() -> crate::deferred_log::DeferredEntry {
    build_display_log_entry(
        "[malvin.........] heartbeat".into(),
        "20260524.000000.000 [malvin.........] heartbeat".into(),
    )
}

fn sink_queue_heartbeat_fixture() -> (SharedDeferSink, String, String) {
    let shared = aged_defer_shared("stale_flag");
    let (display, log_line) = crate::output::stdout_tagged_display_and_log_line(
        crate::output::MALVIN_WHO,
        "heartbeat",
        Some("20260524.000000.000"),
    );
    (shared, display, log_line)
}

#[test]
fn try_log_under_mutex_bundles_due_heartbeat_when_none_deferred() {
    let _stdout_guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _heartbeat_guard = crate::output::HEARTBEAT_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    crate::output::reset_stdout_heartbeat_for_test();
    crate::output::test_set_last_heartbeat_elapsed(std::time::Duration::from_secs(61));
    crate::deferred_log::install_stdout_hooks();
    let shared = zero_age_defer_shared("bundle_due");
    register(Arc::clone(&shared));
    {
        let _hold = shared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(try_log(build_display_log_entry(
            "CONTENDED_TAG".into(),
            "CONTENDED_TAG".into(),
        )));
        assert_eq!(
            super::pending_len(),
            2,
            "due heartbeat must bundle with tagged entry when nothing is already deferred"
        );
    }
    unregister();
}

#[test]
fn unregister_while_sink_mutex_held_emits_pending_to_terminal() {
    let (terminal, log) = capture_stdout_render_unlocked(|| {
        let shared = zero_age_defer_shared("unreg_hold");
        register(Arc::clone(&shared));
        {
            let _hold = shared
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            assert!(try_log(build_display_log_entry(
                "LOST_TAG".into(),
                "LOST_TAG".into(),
            )));
            assert_eq!(super::pending_len(), 1);
            unregister();
        }
        assert_eq!(
            super::pending_len(),
            0,
            "unregister must flush and clear pending even when sink mutex was held"
        );
    });
    assert!(
        terminal.contains("LOST_TAG") || log.contains("LOST_TAG"),
        "pending deferred during contended unregister must reach terminal or log"
    );
}

fn run_stale_flag_contention_scenario(
    shared: &SharedDeferSink,
    display: &str,
    log_line: &str,
) -> (String, String) {
    capture_stdout_render_unlocked(|| {
        register(Arc::clone(shared));
        crate::output::write_heartbeat_log_line(display, log_line);
        assert_eq!(
            shared
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .queue_len(),
            1
        );
        {
            let _hold = shared
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            unregister();
            register(Arc::clone(shared));
            assert!(try_log(build_display_log_entry(
                "STALE_FLAG_TAG".into(),
                "STALE_FLAG_TAG".into(),
            )));
            assert_eq!(
                super::pending_len(),
                1,
                "must not bundle heartbeat when sink queue already holds one after contended unregister"
            );
        }
        shared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .force_flush();
    })
}

#[test]
#[allow(unsafe_code)]
fn contended_unregister_stale_flag_bundles_duplicate_heartbeat() {
    unsafe {
        std::env::set_var("MALVIN_DEFER_LOG_MAX_AGE_MS", "3600000");
    }
    let _stdout_guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _heartbeat_guard = crate::output::HEARTBEAT_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    crate::output::reset_stdout_heartbeat_for_test();
    crate::output::test_set_last_heartbeat_elapsed(std::time::Duration::from_secs(61));
    crate::deferred_log::install_stdout_hooks();
    let (shared, display, log_line) = sink_queue_heartbeat_fixture();
    let (terminal, log) = run_stale_flag_contention_scenario(&shared, &display, &log_line);
    unsafe {
        std::env::remove_var("MALVIN_DEFER_LOG_MAX_AGE_MS");
    }
    assert_eq!(
        terminal.lines().filter(|l| l.contains("heartbeat")).count(),
        1
    );
    assert_eq!(
        log.lines().filter(|l| l.contains("heartbeat")).count(),
        1
    );
}

#[test]
fn contended_try_push_dedupes_explicit_heartbeat() {
    let _stdout_guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let shared = zero_age_defer_shared("try_push_dedup");
    register(Arc::clone(&shared));
    {
        let _hold = shared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(try_push(heartbeat_entry()));
        assert!(try_push(heartbeat_entry()));
        assert_eq!(
            super::pending_len(),
            1,
            "contended try_push must not queue duplicate explicit heartbeats"
        );
    }
    unregister();
}
