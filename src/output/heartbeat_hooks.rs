use std::sync::OnceLock;

pub struct HeartbeatHooks {
    pub payload: fn() -> String,
    pub notify_working: fn(),
}

static HOOKS: OnceLock<HeartbeatHooks> = OnceLock::new();

pub fn install_heartbeat_hooks(hooks: HeartbeatHooks) {
    let _ = HOOKS.set(hooks);
}

pub(crate) fn heartbeat_payload() -> String {
    HOOKS
        .get()
        .map_or_else(crate::time_format::heartbeat_timestamp, |hooks| {
            (hooks.payload)()
        })
}

pub(crate) fn notify_working() {
    if let Some(hooks) = HOOKS.get() {
        (hooks.notify_working)();
    }
}
