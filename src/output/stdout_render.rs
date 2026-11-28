#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum StdoutRenderPrelude {
    TaggedWithHeartbeat,
    HeartbeatOnly,
    FlushOnly,
}

pub(crate) use super::stdout_emit::emit_stdout_rendered_immediate;
#[cfg(test)]
pub(crate) use super::stdout_heartbeat::write_heartbeat_log_line;

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
