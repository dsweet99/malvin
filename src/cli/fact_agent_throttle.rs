use std::path::{Path, PathBuf};

pub(crate) const FACT_AGENT_INTERVAL_SECS: u64 = 60 * 60;

pub(crate) fn record_path() -> PathBuf {
    malvin::workspace::workspace_paths::malvin_user_home_root().join("last_fact_agent_run.json")
}

fn load_last_run_secs(path: &Path) -> Option<u64> {
    let body = std::fs::read_to_string(path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&body).ok()?;
    value.get("last_run_secs")?.as_u64()
}

pub(crate) fn is_due(path: &Path, now_secs: u64) -> bool {
    load_last_run_secs(path)
        .is_none_or(|last| now_secs.saturating_sub(last) >= FACT_AGENT_INTERVAL_SECS)
}

fn save_last_run_secs(path: &Path, now_secs: u64) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let body = serde_json::json!({ "last_run_secs": now_secs }).to_string();
    std::fs::write(path, body).map_err(|e| format!("write {}: {e}", path.display()))
}

pub(crate) fn try_claim(path: &Path, now_secs: u64) -> bool {
    if !is_due(path, now_secs) {
        return false;
    }
    save_last_run_secs(path, now_secs).is_ok()
}

#[cfg(test)]
#[path = "fact_agent_throttle_tests.rs"]
mod fact_agent_throttle_tests;
