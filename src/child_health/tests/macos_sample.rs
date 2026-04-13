use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use super::*;

fn state_hint_ok(h: &ChildHealth) -> bool {
    h.state_hint
        .is_none_or(|c| matches!(c, 'R' | 'S' | 'T' | 'Z'))
}

#[test]
fn sample_live_sleep_child_reports_running_shape() {
    let mut child = Command::new("/bin/sleep")
        .arg("60")
        .spawn()
        .expect("spawn sleep");
    let pid = child.id();
    let h = sample_child_health(pid);
    assert!(h.exists, "expected live child: {h:?}");
    assert!(
        h.counters_trusted,
        "macOS proc_pidinfo should yield trusted counters: {h:?}"
    );
    assert!(
        h.thread_count.is_some(),
        "thread count should be available from task info: {h:?}"
    );
    assert!(
        h.sample_time.elapsed() < Duration::from_secs(2),
        "sample_time should be a fresh clock: {h:?}"
    );
    assert!(
        state_hint_ok(&h),
        "unexpected state hint for live child: {h:?}"
    );
    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn two_live_child_samples_are_ordered_trusted_snapshots() {
    let mut child = Command::new("/bin/sleep")
        .arg("120")
        .spawn()
        .expect("spawn sleep");
    let pid = child.id();
    let first = sample_child_health(pid);
    thread::sleep(Duration::from_millis(40));
    let second = sample_child_health(pid);
    assert!(first.exists && second.exists);
    assert!(first.counters_trusted && second.counters_trusted);
    assert!(
        first.sample_time < second.sample_time,
        "two sequential samples should reflect ordering across the delay: {first:?} then {second:?}"
    );
    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn sample_after_child_exit_is_absent_or_safe_non_running() {
    let mut child = Command::new("/usr/bin/true").spawn().expect("spawn true");
    let pid = child.id();
    let status = child.wait().expect("wait");
    assert!(status.success());
    let h = sample_child_health(pid);
    if h.exists {
        // The PID slot may already name another live process; we only require no panic here.
    } else {
        assert!(
            h.counters_trusted,
            "`process_absent` uses trusted placeholder counters: {h:?}"
        );
        assert!(!h.zombie, "absent PID must not read as zombie: {h:?}");
    }
}

#[test]
fn health_indicates_progress_with_macos_shaped_pair() {
    let t0 = Instant::now();
    let before = ChildHealth {
        exists: true,
        zombie: false,
        state_hint: Some('R'),
        counters_trusted: true,
        cpu_time_total: 100,
        thread_count: Some(4),
        voluntary_ctxt: Some(10),
        sample_time: t0,
    };
    let after_cpu = ChildHealth {
        cpu_time_total: 101,
        exists: true,
        zombie: false,
        state_hint: Some('R'),
        counters_trusted: true,
        thread_count: Some(4),
        voluntary_ctxt: Some(10),
        sample_time: t0,
    };
    assert!(health_indicates_progress(&before, &after_cpu));

    let after_ctxt = ChildHealth {
        voluntary_ctxt: Some(11),
        cpu_time_total: 100,
        exists: true,
        zombie: false,
        state_hint: Some('R'),
        counters_trusted: true,
        thread_count: Some(4),
        sample_time: t0,
    };
    assert!(health_indicates_progress(&before, &after_ctxt));

    let stale = ChildHealth {
        cpu_time_total: 100,
        voluntary_ctxt: Some(10),
        thread_count: Some(4),
        exists: true,
        zombie: false,
        state_hint: Some('R'),
        counters_trusted: true,
        sample_time: t0,
    };
    assert!(!health_indicates_progress(&before, &stale));
}
