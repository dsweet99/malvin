pub fn notify_run_done() {
    let _ = std::panic::catch_unwind(notify_run_done_inner);
}

fn notify_run_done_inner() {
    if !super::lifecycle::live_io_allowed() {
        return;
    }
    super::lifecycle::report_active_state("idle");
}

pub(super) const fn terminal_agent_state() -> &'static str {
    "idle"
}
