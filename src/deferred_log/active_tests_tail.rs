use super::*;

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
            let _acp_hold = shared
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
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
