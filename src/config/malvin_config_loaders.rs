use std::collections::BTreeMap;
use std::path::Path;

use crate::config::log_gc_config::LogsGcConfig;
use crate::config::model_id::ParsedModel;

use super::malvin_config_file::load_malvin_config;

const GIB: u64 = 1024 * 1024 * 1024;

pub fn load_logs_gc_config(work_dir: &Path) -> LogsGcConfig {
    load_malvin_config(work_dir).logs
}

#[must_use]
pub fn load_mem_limit_bytes(work_dir: &Path) -> u64 {
    let gb = load_mem_limit_gb(work_dir);
    gb.saturating_mul(GIB)
}

#[must_use]
pub fn load_mem_limit_gb(work_dir: &Path) -> u64 {
    load_malvin_config(work_dir).mem_limit_gb
}

pub fn parse_model_cli_arg(raw: &str) -> Result<ParsedModel, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    load_malvin_config(&cwd).resolve_model(raw)
}

fn home_aliases() -> (BTreeMap<String, String>, BTreeMap<String, String>) {
    let Ok(cwd) = std::env::current_dir() else {
        return (BTreeMap::new(), BTreeMap::new());
    };
    let cfg = load_malvin_config(&cwd);
    (cfg.model_aliases, cfg.remote_aliases)
}

#[must_use]
pub(crate) fn load_model_aliases() -> BTreeMap<String, String> {
    home_aliases().0
}

#[must_use]
pub fn load_remote_aliases() -> BTreeMap<String, String> {
    home_aliases().1
}

pub fn parse_model_list_cli_arg(raw: &str) -> Result<crate::config::model_list::ModelList, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let config = load_malvin_config(&cwd);
    let models = crate::config::model_list::split_model_list(raw)?
        .into_iter()
        .map(|model| config.resolve_model(model))
        .collect::<Result<Vec<_>, _>>()?;
    crate::config::model_list::ModelList::new(models)
}
