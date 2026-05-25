use super::{register, try_log, unregister, SharedDeferSink};
use crate::deferred_log::{
    build_display_log_entry,
    test_fixtures::{aged_defer_shared, capture_stdout_render_unlocked},
};
use std::sync::Arc;

fn sink_queue_contention_fixture() -> (SharedDeferSink, String, String) {
    let shared = aged_defer_shared("sink_q_contention");
    let (display, log_line) = crate::output::stdout_heartbeat_display_and_log_line(
        crate::output::MALVIN_WHO,
        "HB: 20260524.000000",
        Some("20260524.000000.000"),
    );
    (shared, display, log_line)
}

fn try_log_while_sink_mutex_held(shared: &SharedDeferSink) {
    let _hold = shared
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    assert!(try_log(build_display_log_entry(
        "SINK_Q_TAG".into(),
        "SINK_Q_TAG".into(),
    )));
}

#[test]
#[allow(unsafe_code)]
fn try_log_under_mutex_does_not_dup_heartbeat_in_sink_queue() {
    unsafe {
        std::env::set_var("MALVIN_DEFER_LOG_MAX_AGE_MS", "3600000");
    }
    let _heartbeat_guard = crate::output::HEARTBEAT_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    crate::output::reset_stdout_heartbeat_for_test();
    crate::output::test_set_last_heartbeat_elapsed(std::time::Duration::from_secs(61));
    crate::deferred_log::install_stdout_hooks();
    let (shared, display, log_line) = sink_queue_contention_fixture();
    let (terminal, log) = capture_stdout_render_unlocked(|| {
        register(Arc::clone(&shared));
        crate::output::write_heartbeat_log_line(&display, &log_line);
        assert_eq!(
            shared
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .queue_len(),
            1
        );
        try_log_while_sink_mutex_held(&shared);
        assert_eq!(
            super::pending_len(),
            1,
            "must not bundle heartbeat when defer sink queue already holds one"
        );
        unregister();
        shared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .force_flush();
    });
    unsafe {
        std::env::remove_var("MALVIN_DEFER_LOG_MAX_AGE_MS");
    }
    assert_eq!(
        terminal.lines().filter(|l| l.contains("HB:")).count(),
        1
    );
    assert_eq!(
        log.lines().filter(|l| l.contains("HB:")).count(),
        1
    );
}
