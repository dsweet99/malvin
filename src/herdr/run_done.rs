use std::sync::atomic::{AtomicBool, Ordering};

static MARKED_AT_TIMING: AtomicBool = AtomicBool::new(false);

pub fn notify_run_done() {
    let _ = std::panic::catch_unwind(notify_run_done_inner);
}

fn notify_run_done_inner() {
    if !super::lifecycle::live_io_allowed() {
        return;
    }
    MARKED_AT_TIMING.store(true, Ordering::Relaxed);
    super::lifecycle::report_active_state("done");
}

pub(super) fn terminal_agent_state() -> &'static str {
    if MARKED_AT_TIMING.swap(false, Ordering::Relaxed) {
        "done"
    } else {
        "idle"
    }
}
