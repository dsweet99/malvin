use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

pub const MODELS_REFRESH_INTERVAL_SECS: u64 = 24 * 60 * 60;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelsRefreshRecord {
    #[serde(alias = "timestamp")]
    pub last_refresh_secs: u64,
}

#[must_use]
pub fn unix_now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[must_use]
pub fn models_refresh_record_path() -> PathBuf {
    malvin::workspace_paths::malvin_user_home_root().join("last_models_refresh.json")
}

#[must_use]
pub fn load_last_refresh_secs() -> Option<u64> {
    let path = models_refresh_record_path();
    let body = std::fs::read_to_string(path).ok()?;
    let record: ModelsRefreshRecord = serde_json::from_str(&body).ok()?;
    Some(record.last_refresh_secs)
}

pub fn save_last_refresh_secs(now_secs: u64) -> Result<(), String> {
    let path = models_refresh_record_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let record = ModelsRefreshRecord {
        last_refresh_secs: now_secs,
    };
    let json = serde_json::to_string_pretty(&record).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| format!("write {}: {e}", path.display()))
}

#[must_use]
pub fn models_refresh_is_due(now_secs: u64) -> bool {
    load_last_refresh_secs()
        .is_none_or(|last| now_secs.saturating_sub(last) >= MODELS_REFRESH_INTERVAL_SECS)
}

pub fn perform_models_refresh() {
    let now = unix_now_secs();
    let _ = malvin::npm_pi_sdk::refresh_npm_pi_models();
    let _ = malvin::pi_sdk::refresh_pi_provider_caches_if_stale(true);
    let _ = save_last_refresh_secs(now);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_record_with_alias_timestamp() {
        let json = r#"{"timestamp": 123456789}"#;
        let record: ModelsRefreshRecord = serde_json::from_str(json).expect("parse");
        assert_eq!(record.last_refresh_secs, 123_456_789);
    }

    #[test]
    fn parses_record_with_last_refresh_secs() {
        let json = r#"{"last_refresh_secs": 987654321}"#;
        let record: ModelsRefreshRecord = serde_json::from_str(json).expect("parse");
        assert_eq!(record.last_refresh_secs, 987_654_321);
    }

    #[test]
    fn unix_now_secs_is_reasonable() {
        assert!(unix_now_secs() > 1_700_000_000);
    }

    #[test]
    fn models_refresh_due_check_logic() {
        let now = 1_000_000_u64;
        let recent = now - 3600;
        assert!(now.saturating_sub(recent) < MODELS_REFRESH_INTERVAL_SECS);
        let stale = now - (MODELS_REFRESH_INTERVAL_SECS + 10);
        assert!(now.saturating_sub(stale) >= MODELS_REFRESH_INTERVAL_SECS);
    }
}
