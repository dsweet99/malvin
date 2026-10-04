use crate::malvin_config_file::{read_string, read_u64};

pub const MAX_TIMEOUT_H: u64 = 24;
const DEFAULT_CPU: u64 = 2;
const HEADROOM_GB: u64 = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModalConfig {
    pub cpu: u64,
    pub memory_gb: u64,
    pub timeout_h: u64,
    pub image: Option<String>,
    pub setup: Vec<String>,
    pub remote_mem_limit_gb: u64,
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

fn read_positive(section: Option<&toml::Value>, key: &str) -> Result<Option<u64>, String> {
    let Some(raw) = section.and_then(|s| s.get(key)) else {
        return Ok(None);
    };
    match read_u64(Some(raw)) {
        Some(0) | None => Err(format!("[modal] {key} must be a positive integer")),
        Some(v) => Ok(Some(v)),
    }
}

pub fn parse_modal_config(root: &toml::Value, local_mem_limit_gb: u64) -> Result<ModalConfig, String> {
    let section = root.get("modal");
    let timeout_h = read_positive(section, "timeout_h")?.unwrap_or(MAX_TIMEOUT_H);
    if timeout_h > MAX_TIMEOUT_H {
        return Err(format!(
            "[modal] timeout_h is {timeout_h}, but Modal Sandboxes live at most {MAX_TIMEOUT_H} hours"
        ));
    }
    let memory_gb = read_positive(section, "memory_gb")?.unwrap_or(local_mem_limit_gb + HEADROOM_GB);
    Ok(ModalConfig {
        cpu: read_positive(section, "cpu")?.unwrap_or(DEFAULT_CPU),
        memory_gb,
        timeout_h,
        image: read_string(section.and_then(|s| s.get("image"))),
        setup: read_setup(section)?,
        remote_mem_limit_gb: local_mem_limit_gb.min(memory_gb.saturating_sub(HEADROOM_GB).max(1)),
    })
}

pub fn remote_config_text(root: &toml::Value, cfg: &ModalConfig) -> Result<String, String> {
    let mut value = root.clone();
    let table = value
        .as_table_mut()
        .ok_or("config.toml must be a TOML table")?;
    let gb = i64::try_from(cfg.remote_mem_limit_gb).map_err(|e| e.to_string())?;
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
