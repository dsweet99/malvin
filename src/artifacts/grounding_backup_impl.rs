use super::{GroundingBackup, ProtectedWorkspaceFiles};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

pub(super) fn backup_workspace_grounding_if_present_with_id(
    work_dir: &Path,
    mut generate_id: impl FnMut(usize) -> String,
) -> Result<GroundingBackup, String> {
    let grounding_src = work_dir.join("grounding.md");
    let kissconfig_src = work_dir.join(".kissconfig");
    if !grounding_src.is_file() && !kissconfig_src.is_file() {
        return Ok(GroundingBackup::Missing);
    }
    let grounding_root = crate::prompts::user_home_dir()
        .join(".malvin")
        .join("groundings");
    let dest_dir = allocate_grounding_backup_dir(&grounding_root, &mut generate_id)?;
    let (grounding, kissconfig) =
        backup_workspace_files(&grounding_src, &kissconfig_src, &dest_dir)?;
    Ok(GroundingBackup::Present(ProtectedWorkspaceFiles {
        grounding,
        kissconfig,
    }))
}

fn backup_workspace_files(
    grounding_src: &Path,
    kissconfig_src: &Path,
    destination_dir: &Path,
) -> Result<(Option<PathBuf>, Option<PathBuf>), String> {
    let grounding = match backup_workspace_file(grounding_src, destination_dir, "grounding.md") {
        Ok(grounding) => grounding,
        Err(err) => {
            cleanup_partial_backup_dir(destination_dir);
            return Err(err);
        }
    };
    let kissconfig = match backup_workspace_file(kissconfig_src, destination_dir, ".kissconfig") {
        Ok(kissconfig) => kissconfig,
        Err(err) => {
            cleanup_partial_backup_dir(destination_dir);
            return Err(err);
        }
    };
    Ok((grounding, kissconfig))
}

fn allocate_grounding_backup_dir(
    grounding_root: &Path,
    generate_id: &mut impl FnMut(usize) -> String,
) -> Result<PathBuf, String> {
    std::fs::create_dir_all(grounding_root).map_err(|e| format!("grounding backup mkdir: {e}"))?;
    let mut tries = 0usize;
    while tries < 16 {
        let candidate = grounding_root.join(generate_id(tries));
        match std::fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(err) if err.kind() == ErrorKind::AlreadyExists => {
                tries += 1;
            }
            Err(err) => return Err(format!("grounding backup mkdir: {err}")),
        }
    }
    Err("grounding backup mkdir: too many id collisions".to_string())
}

fn backup_workspace_file(
    source: &Path,
    destination_dir: &Path,
    filename: &str,
) -> Result<Option<PathBuf>, String> {
    if !source.is_file() {
        return Ok(None);
    }
    let destination = destination_dir.join(filename);
    std::fs::copy(source, &destination)
        .map_err(|e| format!("{filename} backup copy: {e}"))
        .map(|_| Some(destination))
}

fn cleanup_partial_backup_dir(dest_dir: &Path) {
    let _ = std::fs::remove_dir_all(dest_dir);
}
