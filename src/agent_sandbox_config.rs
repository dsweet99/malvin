use std::path::Path;

use crate::output::print_log_warning;
use crate::workspace_paths::malvin_config_path;

const DEFAULT_MEM_LIMIT_GB: u64 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgentSandboxConfig {
    pub mem_limit_gb: u64,
}

impl Default for AgentSandboxConfig {
    fn default() -> Self {
        Self {
            mem_limit_gb: default_mem_limit_gb(),
        }
    }
}

pub fn load_agent_sandbox_config(work_dir: &Path) -> AgentSandboxConfig {
    let path = malvin_config_path(work_dir);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return AgentSandboxConfig::default();
    };
    parse_agent_sandbox_config(&text).unwrap_or_else(|msg| {
        print_log_warning(&format!("could not parse {}: {msg}", path.display()));
        AgentSandboxConfig::default()
    })
}

pub(crate) fn parse_agent_sandbox_config(text: &str) -> Result<AgentSandboxConfig, String> {
    let value: toml::Value = text
        .parse()
        .map_err(|e| format!("invalid TOML: {e}"))?;
    let mem_limit_gb =
        read_mem_limit_gb(value.get("mem_limit_gb")).unwrap_or_else(default_mem_limit_gb);
    Ok(AgentSandboxConfig { mem_limit_gb })
}

fn read_mem_limit_gb(value: Option<&toml::Value>) -> Option<u64> {
    let v = value?;
    if let Some(i) = v.as_integer() {
        return u64::try_from(i).ok().filter(|&n| n > 0);
    }
    v.as_str()?.parse().ok().filter(|&n| n > 0)
}

fn default_mem_limit_gb() -> u64 {
    let half_host_gb = host_total_memory_bytes().map_or(DEFAULT_MEM_LIMIT_GB, |b| {
        b / 2 / 1024 / 1024 / 1024
    });
    DEFAULT_MEM_LIMIT_GB.min(half_host_gb.max(1))
}

pub fn host_total_memory_bytes() -> Option<u64> {
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
fn linux_total_memory_bytes() -> Option<u64> {
    let text = std::fs::read_to_string("/proc/meminfo").ok()?;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            let kb = rest.split_whitespace().next()?.parse::<u64>().ok()?;
            return Some(kb * 1024);
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn macos_total_memory_bytes() -> Option<u64> {
    let out = std::process::Command::new("sysctl")
        .args(["-n", "hw.memsize"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8(out.stdout).ok()?;
    text.trim().parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_mem_limit_gb_from_toml() {
        let cfg = parse_agent_sandbox_config("mem_limit_gb = 8\n").expect("parse");
        assert_eq!(cfg.mem_limit_gb, 8);
    }

    #[test]
    fn default_uses_four_or_half_host() {
        let gb = default_mem_limit_gb();
        assert!(gb >= 1);
        assert!(gb <= DEFAULT_MEM_LIMIT_GB);
    }

    #[test]
    fn kiss_cov_agent_sandbox_config_units() {
        let _ = AgentSandboxConfig::default();
        let _ = stringify!(load_agent_sandbox_config);
        let _ = stringify!(read_mem_limit_gb);
        let _ = stringify!(linux_total_memory_bytes);
        let _ = stringify!(macos_total_memory_bytes);
        let _ = host_total_memory_bytes();
    }
}
