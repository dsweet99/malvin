use std::collections::BTreeMap;

use serde_json::{Value, json};

use super::bridge::ModalBridge;

pub const TAG_RUN: &str = "malvin-run";
pub const TAG_HOST: &str = "malvin-host";
pub const TAG_PID: &str = "malvin-pid";

#[must_use]
pub fn host_name() -> String {
    let from_proc = std::fs::read_to_string("/proc/sys/kernel/hostname").ok();
    let name = from_proc.or_else(|| {
        std::process::Command::new("uname")
            .arg("-n")
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
    });
    name.map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| "unknown-host".to_string())
}

#[must_use]
pub fn run_tags(run_id: &str, host: &str, pid: u32) -> BTreeMap<String, String> {
    BTreeMap::from([
        (TAG_RUN.to_string(), run_id.to_string()),
        (TAG_HOST.to_string(), host.to_string()),
        (TAG_PID.to_string(), pid.to_string()),
    ])
}

#[must_use]
pub fn stale_ids(listed: &Value, host: &str, alive: impl Fn(u32) -> bool) -> Vec<String> {
    let Some(items) = listed["sandboxes"].as_array() else {
        return Vec::new();
    };
    items
        .iter()
        .filter(|sb| sb["tags"][TAG_HOST].as_str() == Some(host))
        .filter(|sb| {
            sb["tags"][TAG_PID]
                .as_str()
                .and_then(|p| p.parse::<u32>().ok())
                .is_none_or(|pid| !alive(pid))
        })
        .filter_map(|sb| sb["sandbox_id"].as_str().map(str::to_string))
        .collect()
}

#[cfg(unix)]
fn pid_alive(pid: u32) -> bool {
    crate::acp::pid_alive(pid)
}

#[cfg(not(unix))]
fn pid_alive(_pid: u32) -> bool {
    true
}

pub fn sweep_stale(bridge: &mut ModalBridge, host: &str) -> Result<usize, String> {
    let listed = bridge.call("list_tagged", json!({ "tags": { TAG_HOST: host } }))?;
    let ids = stale_ids(&listed, host, pid_alive);
    if ids.is_empty() {
        return Ok(0);
    }
    bridge.call("terminate_ids", json!({ "ids": ids }))?;
    Ok(ids.len())
}
