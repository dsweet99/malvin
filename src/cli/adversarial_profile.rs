//! Adversarial profile detection for `malvin plan` prompt overlays.

use std::path::{Path, PathBuf};

pub const SMELL_REGISTRY_FILE: &str = "smell_registry.toml";

fn path_matches_adversarial_glob(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    name.contains("adversarial") || name.contains("adv_system")
}

/// True when plan path or workspace triggers adversarial prompt overlays.
#[must_use]
pub fn adversarial_profile_active(plan_path: &Path, work_dir: &Path) -> bool {
    path_matches_adversarial_glob(plan_path) || work_dir.join(SMELL_REGISTRY_FILE).is_file()
}

pub fn adversarial_overlay_hint(plan_path: &Path, work_dir: &Path) -> Option<String> {
    if !adversarial_profile_active(plan_path, work_dir) {
        return None;
    }
    let mut reasons = Vec::new();
    if path_matches_adversarial_glob(plan_path) {
        reasons.push(format!("plan path `{}` matches adversarial glob", plan_path.display()));
    }
    if work_dir.join(SMELL_REGISTRY_FILE).is_file() {
        reasons.push(format!(
            "`{SMELL_REGISTRY_FILE}` exists in {}",
            work_dir.display()
        ));
    }
    Some(reasons.join("; "))
}

#[must_use]
pub fn resolve_work_dir_for_plan(plan_path: &Path) -> PathBuf {
    plan_path.parent().filter(|p| !p.as_os_str().is_empty()).map_or_else(
        || PathBuf::from("."),
        std::path::Path::to_path_buf,
    )
}

#[cfg(test)]
#[path = "adversarial_profile_tests.rs"]
mod adversarial_profile_tests;
