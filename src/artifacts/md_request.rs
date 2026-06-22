use std::path::{Path, PathBuf};

use super::work_dir_for_path;

fn md_path_has_invalid_characters(s: &str) -> bool {
    if s.contains('\0') {
        return true;
    }
    #[cfg(windows)]
    if s.contains(['<', '>', ':', '"', '|', '?', '*']) {
        return true;
    }
    false
}

fn md_path_has_invalid_components(path: &Path) -> bool {
    path.components().any(|c| {
        matches!(
            c,
            std::path::Component::CurDir | std::path::Component::ParentDir
        )
    })
}

/// True when `arg` syntactically names a `.md` path (case-sensitive suffix, no whitespace).
#[must_use]
#[allow(clippy::case_sensitive_file_extension_comparisons)]
pub fn looks_like_md_file_path_arg(arg: &str) -> bool {
    let trimmed = arg.trim();
    !trimmed.is_empty()
        && !trimmed.chars().any(char::is_whitespace)
        && trimmed.ends_with(".md")
        && !md_path_has_invalid_characters(trimmed)
        && !md_path_has_invalid_components(Path::new(trimmed))
}

/// True when `arg` names an existing `.md` file (case-sensitive suffix, no whitespace).
#[must_use]
#[allow(clippy::case_sensitive_file_extension_comparisons)]
pub fn is_existing_md_file_path(arg: &str) -> Option<PathBuf> {
    if !looks_like_md_file_path_arg(arg) {
        return None;
    }
    let trimmed = arg.trim();
    let path = Path::new(trimmed);
    let cwd = std::env::current_dir().ok()?;
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    };
    resolved.is_file().then_some(resolved)
}

/// Resolve CLI request for `code` / `plan`: existing `.md` path reads file; else literal text.
///
/// # Errors
///
/// Returns a message when a matched `.md` file cannot be read.
pub fn resolve_user_md_request(arg: &str) -> Result<(String, PathBuf), String> {
    if let Some(path) = is_existing_md_file_path(arg) {
        let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        return Ok((text, work_dir_for_path(&path)));
    }
    Ok((arg.trim().to_string(), PathBuf::from(".")))
}
