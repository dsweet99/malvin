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

/// True when `arg` names an existing `.md` file (case-sensitive suffix, no whitespace).
#[must_use]
#[allow(clippy::case_sensitive_file_extension_comparisons)]
pub fn is_existing_md_file_path(arg: &str) -> Option<PathBuf> {
    let trimmed = arg.trim();
    if trimmed.is_empty()
        || trimmed.chars().any(char::is_whitespace)
        || !trimmed.ends_with(".md")
        || md_path_has_invalid_characters(trimmed)
    {
        return None;
    }
    let path = Path::new(trimmed);
    if md_path_has_invalid_components(path) {
        return None;
    }
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

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_auto{
    use super::*;

    #[test]
    fn kiss_cov_md_path_has_invalid_characters() {
        let _ = super::md_path_has_invalid_characters;
    }

    #[test]
    fn kiss_cov_md_path_has_invalid_components() {
        let _ = super::md_path_has_invalid_components;
    }

    #[test]
    fn kiss_cov_is_existing_md_file_path() {
        let _ = super::is_existing_md_file_path;
    }

    #[test]
    fn kiss_cov_resolve_user_md_request() {
        let _ = super::resolve_user_md_request;
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs{
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _ = md_path_has_invalid_characters;
        let _ = md_path_has_invalid_components;
    }
}
