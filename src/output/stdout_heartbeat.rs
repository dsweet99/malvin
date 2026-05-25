use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

#[cfg(test)]
use std::sync::atomic::{AtomicBool, Ordering};

use super::{
    MALVIN_WHO, stdout_heartbeat_display_and_log_line, timestamp_now_string,
    write_heartbeat_log_line,
};
use crate::time_format::heartbeat_payload_now;

pub(crate) fn is_heartbeat_log_line(log: &str) -> bool {
    if log.contains("] HB:") {
        return true;
    }
    log.split("] ")
        .nth(1)
        .is_some_and(crate::time_format::heartbeat_payload_has_wall_clock_prefix)
}

pub(crate) fn log_contains_heartbeat(text: &str) -> bool {
    heartbeat_log_offset(text).is_some()
}

pub(crate) fn heartbeat_log_offset(text: &str) -> Option<usize> {
    let mut offset = 0usize;
    for line in text.lines() {
        if is_heartbeat_log_line(line) {
            return Some(offset);
        }
        offset += line.len() + 1;
    }
    None
}

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(60);

#[cfg(test)]
const HEARTBEAT_POLL_INTERVAL: Duration = Duration::from_millis(10);
#[cfg(not(test))]
const HEARTBEAT_POLL_INTERVAL: Duration = Duration::from_secs(1);

static LAST_HEARTBEAT: Mutex<Option<Instant>> = Mutex::new(None);
static WALL_CLOCK_POLLER: OnceLock<()> = OnceLock::new();

#[cfg(test)]
pub(crate) static HEARTBEAT_TEST_LOCK: Mutex<()> = Mutex::new(());

pub(crate) fn mark_heartbeat_emitted(now: Instant) {
    *LAST_HEARTBEAT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(now);
}

pub(crate) fn heartbeat_rendered_if_due(now: Instant, arm_if_unarmed: bool) -> Option<(String, String)> {
    let mut guard = LAST_HEARTBEAT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if guard.is_none() {
        if arm_if_unarmed {
            *guard = Some(now);
        }
        return None;
    }
    let last = guard.expect("armed heartbeat");
    if !heartbeat_due(last, now) {
        return None;
    }
    drop(guard);
    let ts = timestamp_now_string();
    let payload = heartbeat_payload_now();
    Some(stdout_heartbeat_display_and_log_line(
        MALVIN_WHO,
        &payload,
        Some(ts.as_str()),
    ))
}

pub(crate) fn heartbeat_due(last: Instant, now: Instant) -> bool {
    now.checked_duration_since(last)
        .is_some_and(|elapsed| elapsed >= HEARTBEAT_INTERVAL)
}

pub(crate) fn try_emit_heartbeat_if_due(now: Instant, arm_if_unarmed: bool) {
    let Some((display, log)) = heartbeat_rendered_if_due(now, arm_if_unarmed) else {
        return;
    };
    write_heartbeat_log_line(&display, &log);
}

pub(crate) fn maybe_emit_stdout_heartbeat() {
    try_emit_heartbeat_if_due(Instant::now(), true);
}

pub(crate) fn poll_wall_clock_heartbeat_if_due() {
    try_emit_heartbeat_if_due(Instant::now(), false);
}

#[cfg(test)]
static WALL_CLOCK_POLLER_STOP: AtomicBool = AtomicBool::new(false);

pub(crate) fn wall_clock_poller_loop() {
    loop {
        #[cfg(test)]
        if WALL_CLOCK_POLLER_STOP.load(Ordering::Relaxed) {
            break;
        }
        std::thread::sleep(HEARTBEAT_POLL_INTERVAL);
        poll_wall_clock_heartbeat_if_due();
    }
}

#[cfg(test)]
pub(crate) const fn spawn_wall_clock_poller_if_needed() {}

#[cfg(not(test))]
pub(crate) fn spawn_wall_clock_poller_if_needed() {
    WALL_CLOCK_POLLER.get_or_init(|| {
        std::thread::spawn(wall_clock_poller_loop);
    });
}

#[cfg(test)]
pub(crate) fn reset_stdout_heartbeat_for_test() {
    *LAST_HEARTBEAT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
}

#[cfg(test)]
pub(crate) fn test_set_last_heartbeat_elapsed(elapsed: Duration) {
    let last = Instant::now()
        .checked_sub(elapsed)
        .expect("elapsed heartbeat offset must not exceed now");
    *LAST_HEARTBEAT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(last);
}

#[cfg(test)]
mod inline_tests {
    use super::{
        heartbeat_log_offset, heartbeat_rendered_if_due, is_heartbeat_log_line,
        log_contains_heartbeat, mark_heartbeat_emitted, reset_stdout_heartbeat_for_test,
        test_set_last_heartbeat_elapsed, wall_clock_poller_loop, WALL_CLOCK_POLLER_STOP,
        HEARTBEAT_POLL_INTERVAL,
    };
    use std::sync::atomic::Ordering;
    use std::time::{Duration, Instant};

    #[test]
    fn mark_heartbeat_emitted_prevents_immediate_rerender() {
        reset_stdout_heartbeat_for_test();
        test_set_last_heartbeat_elapsed(Duration::from_secs(61));
        mark_heartbeat_emitted(Instant::now());
        assert!(heartbeat_rendered_if_due(Instant::now(), false).is_none());
    }

    #[test]
    fn wall_clock_poller_loop_exits_when_test_stop_is_set() {
        reset_stdout_heartbeat_for_test();
        WALL_CLOCK_POLLER_STOP.store(false, Ordering::Relaxed);
        let handle = std::thread::spawn(wall_clock_poller_loop);
        std::thread::sleep(HEARTBEAT_POLL_INTERVAL + Duration::from_millis(5));
        WALL_CLOCK_POLLER_STOP.store(true, Ordering::Relaxed);
        handle.join().expect("wall clock poller thread");
    }

    #[test]
    fn heartbeat_line_detectors_cover_legacy_and_new_payloads() {
        assert!(is_heartbeat_log_line(
            "20260524.000000.000 [malvin.........] HB: 20260524.000000"
        ));
        assert!(is_heartbeat_log_line(
            "20260524.000000.000 [malvin.........] 20260524.000000 Still alive."
        ));
        assert!(log_contains_heartbeat(
            "20260524.000000.000 [malvin.........] 20260524.000000 Still alive."
        ));
        assert!(!log_contains_heartbeat("plain agent line"));
        assert_eq!(
            heartbeat_log_offset("QUEUED\n20260524.000000.000 [malvin.........] 20260524.000000 Still alive."),
            Some(7)
        );
    }
}
