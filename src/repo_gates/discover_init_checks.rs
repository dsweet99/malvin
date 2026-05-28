//! Deterministic `.malvin/checks` discovery from repo signals for `malvin init`.

use std::path::Path;

use super::init_discovery_validate::validate_checks_command_lines;
use super::{KISS_CHECK_COMMAND};
use crate::malvin_checks_path;

pub use super::discover_init_checks_signals::{
    dedupe_check_lines, discover_init_check_commands, makefile_gate_targets,
    precommit_hook_entries,
};

/// Rewrite `.malvin/checks` from repo signals after init discovery `KPop`.
pub fn finalize_init_checks_from_repo(root: &Path) -> Result<(), String> {
    let lines = discover_init_check_commands(root);
    validate_checks_command_lines(root, &lines)?;
    let checks_path = malvin_checks_path(root);
    if let Some(parent) = checks_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
    }
    let mut content = lines.join("\n");
    if !content.is_empty() {
        content.push('\n');
    }
    std::fs::write(&checks_path, content)
        .map_err(|e| format!("write {}: {e}", checks_path.display()))?;
    Ok(())
}

/// Whether persisted checks cover deduplicated pre-commit hook entries (ignoring kiss).
#[must_use]
pub fn checks_cover_precommit_signals(root: &Path, lines: &[String]) -> bool {
    let expected = dedupe_check_lines(&precommit_hook_entries(root));
    if expected.is_empty() {
        return true;
    }
    let have: std::collections::HashSet<String> = lines
        .iter()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && l.trim() != KISS_CHECK_COMMAND)
        .collect();
    expected
        .iter()
        .all(|cmd| have.contains(cmd.trim()))
}
