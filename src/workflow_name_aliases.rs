//! Legacy workflow and config name aliases after meta→adaptix and adaptix→inspire renames.

use std::path::{Path, PathBuf};

/// Workspace config paths to try when reading, newest canonical path first.
pub const WORKSPACE_CONFIG_PATHS: &[&str] = &[
    ".malvin/config.toml",
    ".malvin/adaptix.toml",
    ".malvin/meta.toml",
];

/// Map a CLI subcommand name to its canonical workflow name.
#[must_use]
pub fn canonical_workflow_name(name: &str) -> &str {
    match name {
        "adaptix" => "inspire",
        other => other,
    }
}

/// Resolve an on-disk session transcript path for `workflow`, checking legacy log names.
#[must_use]
pub fn resolve_session_log_path(run_dir: &Path, workflow: &str) -> PathBuf {
    let canonical = canonical_workflow_name(workflow);
    let primary = run_dir.join(format!("{canonical}.log"));
    if primary.is_file() {
        return primary;
    }
    if canonical == "inspire" {
        let legacy = run_dir.join("adaptix.log");
        if legacy.is_file() {
            return legacy;
        }
    }
    primary
}

/// Resolve workspace metadata config for reads (never for creates).
#[must_use]
pub fn resolve_workspace_malvin_config_path(work_dir: &Path) -> PathBuf {
    for rel in WORKSPACE_CONFIG_PATHS {
        let path = work_dir.join(rel);
        if path.is_file() {
            return path;
        }
    }
    work_dir.join(WORKSPACE_CONFIG_PATHS[0])
}
