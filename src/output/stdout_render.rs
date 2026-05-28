#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StdoutRenderPrelude {
    TaggedWithHeartbeat,
    HeartbeatOnly,
    FlushOnly,
}

pub(crate) fn emit_stdout_rendered_immediate(display: &str, log: &str) {
    super::print_stdout_display_line(display);
    super::append_stdout_log_line(log);
}

/// Heartbeat display on the live terminal while defer keeps log lines in FIFO order.
pub(crate) fn publish_heartbeat_live_terminal(display: &str) {
    super::print_stdout_display_line(display);
    super::stdout_heartbeat::mark_heartbeat_emitted(std::time::Instant::now());
}

pub(crate) fn route_stdout_rendered_line(display: &str, log: &str, prelude: StdoutRenderPrelude) {
    let deferred = match prelude {
        StdoutRenderPrelude::FlushOnly => false,
        StdoutRenderPrelude::TaggedWithHeartbeat => super::stdout_defer::try_defer_tagged_stdout(display, log),
        StdoutRenderPrelude::HeartbeatOnly => super::stdout_defer::try_defer_heartbeat(display, log),
    };
    if deferred {
        return;
    }
    if prelude == StdoutRenderPrelude::TaggedWithHeartbeat {
        super::stdout_heartbeat::maybe_emit_stdout_heartbeat();
    }
    emit_stdout_rendered_immediate(display, log);
    if prelude == StdoutRenderPrelude::HeartbeatOnly {
        super::stdout_heartbeat::mark_heartbeat_emitted(std::time::Instant::now());
    }
}

pub(crate) fn print_stdout_rendered_line(display: &str, log: &str) {
    route_stdout_rendered_line(display, log, StdoutRenderPrelude::TaggedWithHeartbeat);
}

pub(crate) fn write_heartbeat_log_line(display: &str, log: &str) {
    route_stdout_rendered_line(display, log, StdoutRenderPrelude::HeartbeatOnly);
}

pub(crate) fn flush_stdout_rendered_line(display: &str, log: &str) {
    route_stdout_rendered_line(display, log, StdoutRenderPrelude::FlushOnly);
}
