use crate::output::heartbeat_hooks::{HeartbeatHooks, install_heartbeat_hooks};
use crate::time_format::{HEARTBEAT_FIELD_SEPARATOR, heartbeat_timestamp};

#[must_use]
pub fn heartbeat_payload_now() -> String {
    let mut payload = heartbeat_timestamp();
    if let Some(stats) =
        crate::agent_process::active_agent_heartbeat::active_agent_heartbeat_stats()
    {
        payload.push_str(", ");
        payload.push_str(&stats);
    }
    if let Some(id) = crate::workspace::run_id::active_run_dir()
        .as_deref()
        .and_then(crate::workspace::run_id::short_malvin_log_id)
    {
        payload.push_str(HEARTBEAT_FIELD_SEPARATOR);
        payload.push_str(&id);
        let summary = crate::task_summary::current();
        if !summary.is_empty() {
            payload.push_str(HEARTBEAT_FIELD_SEPARATOR);
            payload.push_str(&summary);
        }
        if let Some(status) = crate::task_summary::current_status() {
            payload.push_str(HEARTBEAT_FIELD_SEPARATOR);
            payload.push_str(&status);
        }
    }
    payload
}

pub fn install() {
    install_heartbeat_hooks(HeartbeatHooks {
        payload: heartbeat_payload_now,
        notify_working: crate::herdr::notify_working,
    });
}
