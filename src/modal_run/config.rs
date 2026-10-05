use crate::malvin_config_file::read_string;

use super::options::{
    DEFAULT_MEMORY_GB, DEFAULT_NCPU, DEFAULT_TIMEOUT_S, GpuChoice, ModalOptions, parse_gpu,
    parse_memory, parse_ncpu, parse_timeout,
};

const HEADROOM_GB: u64 = 2;
const RENAMED_KEYS: &[(&str, &str)] =
    &[("cpu", "ncpu"), ("timeout_h", "timeout"), ("memory_gb", "mem"), ("memory", "mem")];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModalConfig {
    pub gpu: Option<String>,
    pub ncpu: u64,
    pub memory_gb: u64,
    pub timeout_s: u64,
    pub image: Option<String>,
    pub setup: Vec<String>,
    pub local_mem_limit_gb: u64,
}

impl ModalConfig {
    #[must_use]
    pub fn with_options(mut self, opts: &ModalOptions) -> Self {
        if let Some(gpu) = &opts.gpu {
            self.gpu = gpu.clone().into_option();
        }
        self.ncpu = opts.ncpu.unwrap_or(self.ncpu);
        self.memory_gb = opts.memory_gb.unwrap_or(self.memory_gb);
        self.timeout_s = opts.timeout_s.unwrap_or(self.timeout_s);
        self
    }

    #[must_use]
    pub fn remote_mem_limit_gb(&self) -> u64 {
        self.local_mem_limit_gb
            .min(self.memory_gb.saturating_sub(HEADROOM_GB).max(1))
    }

    #[must_use]
    pub fn describe(&self) -> String {
        let gpu = self.gpu.as_deref().unwrap_or("no GPU");
        let timeout = if self.timeout_s.is_multiple_of(60) {
            format!("{} min", self.timeout_s / 60)
        } else {
            format!("{} s", self.timeout_s)
        };
        format!("{gpu}, {} CPU, {} GiB, timeout {timeout}", self.ncpu, self.memory_gb)
    }
}

fn read_setup(section: Option<&toml::Value>) -> Result<Vec<String>, String> {
    let Some(value) = section.and_then(|s| s.get("setup")) else {
        return Ok(Vec::new());
    };
    let items = value
        .as_array()
        .ok_or("[modal] setup must be a list of Dockerfile lines")?;
    items
        .iter()
        .map(|v| {
            v.as_str()
                .map(str::to_string)
                .ok_or_else(|| "[modal] setup entries must be strings".to_string())
        })
        .collect()
}

fn read_parsed<T>(
    section: Option<&toml::Value>,
    key: &str,
    parse: fn(&str) -> Result<T, String>,
) -> Result<Option<T>, String> {
    let Some(raw) = section.and_then(|s| s.get(key)) else {
        return Ok(None);
    };
    let text = match raw {
        toml::Value::String(s) => s.clone(),
        toml::Value::Integer(i) => i.to_string(),
        _ => return Err(format!("[modal] {key} must be a string or an integer")),
    };
    parse(&text).map(Some).map_err(|e| format!("[modal] {e}"))
}

fn reject_renamed(section: Option<&toml::Value>) -> Result<(), String> {
    for (old, new) in RENAMED_KEYS {
        if section.and_then(|s| s.get(*old)).is_some() {
            return Err(format!("[modal] {old} was renamed to {new}; see `malvin --doc`"));
        }
    }
    Ok(())
}

pub fn parse_modal_config(root: &toml::Value, local_mem_limit_gb: u64) -> Result<ModalConfig, String> {
    let section = root.get("modal");
    reject_renamed(section)?;
    Ok(ModalConfig {
        gpu: read_parsed(section, "gpu", parse_gpu)?.and_then(GpuChoice::into_option),
        ncpu: read_parsed(section, "ncpu", parse_ncpu)?.unwrap_or(DEFAULT_NCPU),
        memory_gb: read_parsed(section, "mem", parse_memory)?.unwrap_or(DEFAULT_MEMORY_GB),
        timeout_s: read_parsed(section, "timeout", parse_timeout)?.unwrap_or(DEFAULT_TIMEOUT_S),
        image: read_string(section.and_then(|s| s.get("image"))),
        setup: read_setup(section)?,
        local_mem_limit_gb,
    })
}

pub fn remote_config_text(root: &toml::Value, cfg: &ModalConfig) -> Result<String, String> {
    let mut value = root.clone();
    let table = value
        .as_table_mut()
        .ok_or("config.toml must be a TOML table")?;
    let gb = i64::try_from(cfg.remote_mem_limit_gb()).map_err(|e| e.to_string())?;
    table.insert("mem_limit_gb".to_string(), toml::Value::Integer(gb));
    toml::to_string(&value).map_err(|e| format!("serialize remote config.toml: {e}"))
}

pub fn read_config_root(path: &std::path::Path) -> Result<toml::Value, String> {
    match std::fs::read_to_string(path) {
        Ok(text) => text
            .parse()
            .map_err(|e| format!("{}: invalid TOML: {e}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Ok(toml::Value::Table(toml::map::Map::new()))
        }
        Err(e) => Err(format!("read {}: {e}", path.display())),
    }
}
