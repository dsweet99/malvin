#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum StdoutRenderPrelude {
    TaggedWithHeartbeat,
    HeartbeatOnly,
    FlushOnly,
}

pub(crate) fn emit_stdout_rendered_immediate(display: &str, log: &str) {
    if !display.is_empty() {
        super::print_stdout_display_line(display);
    }
    if !log.is_empty() {
        super::append_stdout_log_line(log);
    }
}

pub(crate) fn route_stdout_rendered_line(display: &str, log: &str, prelude: StdoutRenderPrelude) {
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
