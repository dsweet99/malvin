use std::path::Path;

use super::emit::emit_repo_gate_line;
use super::types::RepoGateOutput;

pub fn ensure_workspace_style_markers(
    work_dir: &Path,
    output: RepoGateOutput,
) -> Result<(), String> {
    let style_dir = work_dir.join(".malvin_memory");
    if !style_dir.is_dir() {
        std::fs::create_dir_all(&style_dir)
            .map_err(|e| format!("create {}: {e}", style_dir.display()))?;
    }
    touch_if_missing(&style_dir.join("style.md"), output)
}

pub fn touch_if_missing(path: &Path, output: RepoGateOutput) -> Result<(), String> {
    if path.exists() {
        if path.is_file() {
            return Ok(());
        }
        return Err(format!("{} exists but is not a file", path.display()));
    }
    std::fs::File::create(path).map_err(|e| format!("create {}: {e}", path.display()))?;
    emit_repo_gate_line(
        output,
        &format!("Touched empty {} (was missing)", path.display()),
        None,
    );
    Ok(())
}
