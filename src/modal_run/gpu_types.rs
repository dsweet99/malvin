use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::http_fetch::{HttpRequest, fetch_text};

pub const GPU_TYPES_URL: &str = "https://modal.com/docs/guide/gpu.md";
pub const GPU_TYPES_REFRESH_INTERVAL_SECS: u64 = 24 * 60 * 60;
const GPU_TYPES_SECTION: &str = "## Specifying GPU type";
const FETCH_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GpuTypesRecord {
    pub fetched_secs: u64,
    pub types: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GpuTypes {
    pub types: Vec<String>,
    pub note: Option<String>,
}

pub struct GpuTypesSource<'a> {
    pub cache_path: &'a Path,
    pub url: &'a str,
    pub now_secs: u64,
}

#[must_use]
pub fn gpu_types_cache_path() -> PathBuf {
    crate::workspace::workspace_paths::malvin_user_home_root().join("modal_gpu_types.json")
}

#[must_use]
pub fn parse_gpu_types(markdown: &str) -> Vec<String> {
    let Some((_, after)) = markdown.split_once(GPU_TYPES_SECTION) else {
        return Vec::new();
    };
    let section = after.split("\n## ").next().unwrap_or_default();
    section
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix("* ").or_else(|| line.trim_start().strip_prefix("- ")))
        .flat_map(|item| item.split('`').skip(1).step_by(2).map(str::to_string))
        .filter(|t| !t.is_empty())
        .collect()
}

#[must_use]
pub fn load_record(path: &Path) -> Option<GpuTypesRecord> {
    let body = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&body).ok()
}

pub fn save_record(path: &Path, record: &GpuTypesRecord) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(record).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| format!("write {}: {e}", path.display()))
}

fn fetch_record(url: &str, now_secs: u64) -> Option<GpuTypesRecord> {
    let markdown = fetch_text(&HttpRequest::get(url, FETCH_TIMEOUT))?;
    let types = parse_gpu_types(&markdown);
    (!types.is_empty()).then_some(GpuTypesRecord { fetched_secs: now_secs, types })
}

const fn is_fresh(record: &GpuTypesRecord, now_secs: u64) -> bool {
    now_secs.saturating_sub(record.fetched_secs) < GPU_TYPES_REFRESH_INTERVAL_SECS
}

#[must_use]
pub fn load_gpu_types_from(source: &GpuTypesSource<'_>, force: bool) -> GpuTypes {
    let cached = load_record(source.cache_path);
    if let Some(record) = cached.as_ref().filter(|r| !force && is_fresh(r, source.now_secs)) {
        return GpuTypes { types: record.types.clone(), note: None };
    }
    if let Some(record) = fetch_record(source.url, source.now_secs) {
        let _ = save_record(source.cache_path, &record);
        return GpuTypes { types: record.types, note: None };
    }
    let failed = format!("could not fetch GPU types from {}", source.url);
    cached.map_or_else(
        || GpuTypes { types: Vec::new(), note: Some(failed.clone()) },
        |record| {
            let age_h = source.now_secs.saturating_sub(record.fetched_secs) / 3600;
            GpuTypes { types: record.types, note: Some(format!("{failed}; showing the list cached {age_h}h ago")) }
        },
    )
}

#[must_use]
pub fn load_gpu_types(force: bool) -> GpuTypes {
    let path = gpu_types_cache_path();
    let source = GpuTypesSource { cache_path: &path, url: GPU_TYPES_URL, now_secs: crate::clock::unix_now_secs() };
    load_gpu_types_from(&source, force)
}
