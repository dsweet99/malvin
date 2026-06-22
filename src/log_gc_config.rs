use std::path::Path;

use crate::malvin_config_file::read_u64;

const DEFAULT_MAX_AGE_DAYS: u64 = 90;
const DEFAULT_MAX_BYTES: &str = "2GiB";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::struct_field_names)]
pub struct LogsGcConfig {
    pub max_age_days: u64,
    pub max_bytes: Option<u64>,
}

impl Default for LogsGcConfig {
    fn default() -> Self {
        Self {
            max_age_days: DEFAULT_MAX_AGE_DAYS,
            max_bytes: parse_byte_size(DEFAULT_MAX_BYTES),
        }
    }
}

pub fn load_logs_gc_config(work_dir: &Path) -> LogsGcConfig {
    crate::malvin_config_file::load_malvin_config(work_dir).logs
}

pub(crate) fn parse_logs_gc_config(text: &str) -> Result<LogsGcConfig, String> {
    let value: toml::Value = text
        .parse()
        .map_err(|e| format!("invalid TOML: {e}"))?;
    let logs = value.get("logs").ok_or_else(|| "missing [logs] section".to_string())?;
    let max_age_days = read_u64(logs.get("max_age_days")).unwrap_or(DEFAULT_MAX_AGE_DAYS);
    let max_bytes = match logs.get("max_bytes") {
        Some(v) => parse_max_bytes_value(v)?,
        None => parse_byte_size(DEFAULT_MAX_BYTES),
    };
    Ok(LogsGcConfig {
        max_age_days,
        max_bytes,
    })
}

pub(crate) fn parse_max_bytes_value(value: &toml::Value) -> Result<Option<u64>, String> {
    match value {
        toml::Value::String(s) if s.trim().is_empty() => Ok(None),
        toml::Value::String(s) => parse_byte_size(s.trim())
            .map(Some)
            .ok_or_else(|| format!("invalid max_bytes value: {s:?}")),
        _ => Err("max_bytes must be a string".to_string()),
    }
}

pub fn parse_byte_size(raw: &str) -> Option<u64> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    let (num, unit) = split_byte_size(raw)?;
    let n: u64 = num.parse().ok()?;
    n.checked_mul(unit)
}

pub(crate) fn split_byte_size(raw: &str) -> Option<(&str, u64)> {
    const UNITS: [(&str, u64); 9] = [
        ("TiB", 1024_u64.pow(4)),
        ("GiB", 1024_u64.pow(3)),
        ("MiB", 1024_u64.pow(2)),
        ("KiB", 1024),
        ("TB", 1000_u64.pow(4)),
        ("GB", 1000_u64.pow(3)),
        ("MB", 1000_u64.pow(2)),
        ("KB", 1000),
        ("B", 1),
    ];
    for (suffix, mult) in UNITS {
        if let Some(prefix) = raw.strip_suffix(suffix) {
            return Some((prefix.trim(), mult));
        }
    }
    None
}
