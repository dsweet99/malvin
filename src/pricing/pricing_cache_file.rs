use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use super::model_cost::ModelCost;
use serde::{Deserialize, Serialize};

use crate::clock::{SystemClock, fetched_at_is_fresh, unix_now_secs};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct PricedEntry {
    pub(super) fetched_at_secs: u64,
    pub(super) cost: ModelCost,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct CacheBody {
    by_id: HashMap<String, PricedEntry>,
}

pub(super) struct PricingCacheFile {
    file_name: &'static str,
    ttl: Duration,
}

impl PricingCacheFile {
    pub(super) const fn new(file_name: &'static str, ttl: Duration) -> Self {
        Self { file_name, ttl }
    }

    pub(super) fn path(&self) -> PathBuf {
        crate::workspace::workspace_paths::malvin_user_home_root().join(self.file_name)
    }

    pub(super) fn load(&self) -> HashMap<String, PricedEntry> {
        fs::read_to_string(self.path())
            .ok()
            .and_then(|body| serde_json::from_str::<CacheBody>(&body).ok())
            .map(|body| body.by_id)
            .unwrap_or_default()
    }

    fn save(&self, by_id: HashMap<String, PricedEntry>) {
        let path = self.path();
        let temp = path.with_extension(format!("tmp-{}", std::process::id()));
        if let Ok(json) = serde_json::to_string(&CacheBody { by_id }) {
            if fs::write(&temp, json).is_ok() {
                let _ = fs::rename(temp, path);
            } else {
                let _ = fs::remove_file(temp);
            }
        }
    }

    pub(super) fn is_fresh(&self, entry: &PricedEntry) -> bool {
        fetched_at_is_fresh(&SystemClock, entry.fetched_at_secs, self.ttl)
    }

    pub(super) fn fresh_cost(&self, key: &str) -> Option<ModelCost> {
        self.load()
            .remove(key)
            .filter(|entry| self.is_fresh(entry))
            .map(|entry| entry.cost)
    }

    pub(super) fn all_fresh(&self) -> bool {
        let by_id = self.load();
        !by_id.is_empty() && by_id.values().all(|entry| self.is_fresh(entry))
    }

    pub(super) fn insert(&self, key: &str, cost: ModelCost) {
        let mut by_id = self.load();
        by_id.insert(key.to_string(), stamped_now(cost));
        self.save(by_id);
    }

    pub(super) fn replace_all(&self, costs: HashMap<String, ModelCost>) {
        let by_id = costs
            .into_iter()
            .map(|(key, cost)| (key, stamped_now(cost)))
            .collect();
        self.save(by_id);
    }

    #[cfg(test)]
    pub(super) fn save_entries_for_test(&self, by_id: HashMap<String, PricedEntry>) {
        self.save(by_id);
    }
}

fn stamped_now(cost: ModelCost) -> PricedEntry {
    PricedEntry {
        fetched_at_secs: unix_now_secs(),
        cost,
    }
}

#[cfg(test)]
#[path = "pricing_cache_file_tests.rs"]
mod pricing_cache_file_tests;
