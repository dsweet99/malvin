use std::path::PathBuf;

use super::config::ModalConfig;
use super::image::ImageSpec;
use super::remote::RemotePaths;

pub struct Plan {
    pub cfg: ModalConfig,
    pub cfg_root: toml::Value,
    pub spec: ImageSpec,
}

pub struct Prepared {
    pub staging: tempfile::TempDir,
    pub cwd: PathBuf,
    pub home: PathBuf,
    pub logs_root: PathBuf,
    pub paths: RemotePaths,
    pub has_logs: bool,
    pub plan: Plan,
}

pub(crate) fn note(text: &str) {
    super::remote_output::print_status(&format!("modal: {text}"));
}
