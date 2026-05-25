use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use super::{MALVIN_WHO, append_stdout_log_line, format_line_with_timestamp, timestamp_now_string};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(60);

use std::time::Duration;

#[cfg(test)]
const HEARTBEAT_POLL_INTERVAL: Duration = Duration::from_millis(10);
#[cfg(not(test))]
const HEARTBEAT_POLL_INTERVAL: Duration = Duration::from_secs(1);

static LAST_HEARTBEAT: Mutex<Option<Instant>> = Mutex::new(None);
static WALL_CLOCK_POLLER: OnceLock<()> = OnceLock::new();

#[cfg(test)]
pub(crate) static HEARTBEAT_TEST_LOCK: Mutex<()> = Mutex::new(());

fn heartbeat_log_line_if_due(now: Instant, arm_if_unarmed: bool) -> Option<String> {
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
    *guard = Some(now);
    drop(guard);
    let ts = timestamp_now_string();
    Some(format_line_with_timestamp(&ts, MALVIN_WHO, "heartbeat"))
}

pub(crate) fn write_heartbeat_log_line(log_line: &str) {
    if crate::deferred_log::active_defer_sink_registered() {
        if crate::output::stdout_defer::try_defer_push_line(log_line.to_string()) {
            return;
        }
    } else if crate::output::stdout_defer::try_defer_push_line(log_line.to_string()) {
        return;
    }
    println!("{log_line}");
    append_stdout_log_line(log_line);
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn emit_heartbeat_line() {
    let ts = timestamp_now_string();
    write_heartbeat_log_line(&format_line_with_timestamp(&ts, MALVIN_WHO, "heartbeat"));
}

pub(crate) fn heartbeat_due(last: Instant, now: Instant) -> bool {
    now.checked_duration_since(last)
        .is_some_and(|elapsed| elapsed >= HEARTBEAT_INTERVAL)
}

pub(crate) fn try_emit_heartbeat_if_due(now: Instant, arm_if_unarmed: bool) {
    let Some(log_line) = heartbeat_log_line_if_due(now, arm_if_unarmed) else {
        return;
    };
    write_heartbeat_log_line(&log_line);
}

pub(crate) fn maybe_emit_stdout_heartbeat() {
    try_emit_heartbeat_if_due(Instant::now(), true);
}

pub(crate) fn poll_wall_clock_heartbeat_if_due() {
    try_emit_heartbeat_if_due(Instant::now(), false);
}

pub(crate) fn wall_clock_poller_loop() {
    loop {
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

pub(crate) fn heartbeat_log_line_for_defer_sink(
    now: Instant,
    arm_if_unarmed: bool,
) -> Option<String> {
    heartbeat_log_line_if_due(now, arm_if_unarmed)
}
