
use std::path::Path;

use super::{format_quality_gates_markdown, load_malvin_checks};

pub fn prompt_quality_gates_markdown(work_dir: &Path) -> Result<String, String> {
    let checks_path = crate::resolve_malvin_checks_path(work_dir);
    if !checks_path.is_file() {
        return Err(format!(
            "{} is missing (expected a regular file before expanding {{ quality_gates }})",
            checks_path.display()
        ));
    }
    let lines = load_malvin_checks(&checks_path)?;
    Ok(format_quality_gates_markdown(&lines))
}

pub fn prompt_quality_gates_markdown_ephemeral(work_dir: &Path) -> Result<String, String> {
    use crate::session_dotfile_backup::{
        backup_workspace_malvin_checks_if_present, restore_workspace_malvin_checks_backup,
    };
    let backup = backup_workspace_malvin_checks_if_present(work_dir)?;
    let result = prompt_quality_gates_markdown(work_dir);
    restore_workspace_malvin_checks_backup(work_dir, &backup)?;
    result
}
