//! Agent process-group memory cap from `~/.malvin_home/config.toml`.

use std::path::Path;

use crate::malvin_config_file::read_u64;

const GIB: u64 = 1024 * 1024 * 1024;
const DEFAULT_CAP_GB: u64 = 4;

/// RSS cap for an agent process group, in bytes.
#[must_use]
pub fn load_mem_limit_bytes(work_dir: &Path) -> u64 {
    let gb = load_mem_limit_gb(work_dir);
    gb.saturating_mul(GIB)
}

#[must_use]
pub fn load_mem_limit_gb(work_dir: &Path) -> u64 {
    crate::malvin_config_file::load_malvin_config(work_dir).mem_limit_gb
}

pub(crate) fn parse_mem_limit_gb(text: &str) -> Result<u64, String> {
    let value: toml::Value = text
        .parse()
        .map_err(|e| format!("invalid TOML: {e}"))?;
    match read_u64(value.get("mem_limit_gb")) {
        None => Ok(default_mem_limit_gb()),
        Some(0) => Err("mem_limit_gb must be positive".to_string()),
        Some(gb) => Ok(gb),
    }
}

#[must_use]
pub fn default_mem_limit_gb() -> u64 {
    let half_gb = system_total_memory_bytes().map_or(DEFAULT_CAP_GB, |bytes| bytes / 2 / GIB);
    DEFAULT_CAP_GB.min(half_gb.max(1))
}

#[must_use]
pub fn system_cpu_count() -> Option<usize> {
    std::thread::available_parallelism()
        .ok()
        .map(std::num::NonZeroUsize::get)
}

#[must_use]
pub fn format_host_resources_line() -> String {
    let memory = system_total_memory_bytes()
        .map_or_else(|| "unknown".to_string(), format_memory_gib);
    let cpus = system_cpu_count()
        .map_or_else(|| "unknown".to_string(), |n| n.to_string());
    format!("Memory: {memory}, CPUs: {cpus}")
}

#[must_use]
pub fn format_memory_gib(bytes: u64) -> String {
    let gib = bytes / GIB;
    let remainder = bytes % GIB;
    if remainder == 0 || remainder < GIB / 20 {
        format!("{gib} GiB")
    } else {
        let tenths = bytes.saturating_mul(10) / GIB;
        format!("{}.{} GiB", tenths / 10, tenths % 10)
    }
}

#[must_use]
pub fn system_total_memory_bytes() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        linux_total_memory_bytes()
    }
    #[cfg(target_os = "macos")]
    {
        macos_total_memory_bytes()
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        None
    }
}

#[cfg(target_os = "linux")]
pub(crate) fn linux_total_memory_bytes() -> Option<u64> {
    let raw = std::fs::read_to_string("/proc/meminfo").ok()?;
    raw.lines().find_map(|line| {
        let rest = line.strip_prefix("MemTotal:")?;
        let kb_str = rest.trim().strip_suffix(" kB")?.trim();
        let kb: u64 = kb_str.parse().ok()?;
        kb.checked_mul(1024)
    })
}

#[cfg(target_os = "macos")]
pub(crate) fn macos_total_memory_bytes() -> Option<u64> {
    use std::process::Command;
    let out = Command::new("sysctl")
        .arg("-n")
        .arg("hw.memsize")
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8(out.stdout)
        .ok()?
        .trim()
        .parse()
        .ok()
}

#[cfg(test)]
#[path = "mem_limit_config_tests.rs"]
mod mem_limit_config_tests;
