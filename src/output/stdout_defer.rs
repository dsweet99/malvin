use std::sync::OnceLock;

type DeferTaggedStdoutFn = fn(&str, &str) -> bool;
type DeferHeartbeatFn = fn(&str, &str) -> bool;

static DEFER_TAGGED_STDOUT: OnceLock<DeferTaggedStdoutFn> = OnceLock::new();
static DEFER_HEARTBEAT: OnceLock<DeferHeartbeatFn> = OnceLock::new();

pub(crate) fn register_defer_stdout_hooks(tagged: DeferTaggedStdoutFn, heartbeat: DeferHeartbeatFn) {
    let _ = DEFER_TAGGED_STDOUT.set(tagged);
    let _ = DEFER_HEARTBEAT.set(heartbeat);
}

pub(crate) fn try_defer_tagged_stdout(display: &str, log: &str) -> bool {
    DEFER_TAGGED_STDOUT
        .get()
        .is_some_and(|hook| hook(display, log))
}

pub(crate) fn try_defer_heartbeat(display: &str, log: &str) -> bool {
    DEFER_HEARTBEAT
        .get()
        .is_some_and(|hook| hook(display, log))
}

#[cfg(test)]
mod tests {
    use super::{register_defer_stdout_hooks, try_defer_heartbeat, try_defer_tagged_stdout};

    fn noop_tagged(_display: &str, _log: &str) -> bool {
        true
    }

    fn noop_heartbeat(_display: &str, _log: &str) -> bool {
        true
    }

    #[test]
    fn register_defer_stdout_hooks_is_safe_when_called() {
        register_defer_stdout_hooks(noop_tagged, noop_heartbeat);
        let _ = try_defer_tagged_stdout("d", "l");
        let _ = try_defer_heartbeat("hb-d", "hb-l");
    }
}
