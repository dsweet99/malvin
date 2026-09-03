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

fn md_path_is_within_cwd(cwd: &Path, path: &Path) -> bool {
    let Ok(cwd_canon) = cwd.canonicalize() else {
        return false;
    };
    let Ok(path_canon) = path.canonicalize() else {
        return false;
    };
    path_canon == cwd_canon || path_canon.starts_with(&cwd_canon)
}

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
    if !resolved.is_file() {
        return None;
    }
    if !md_path_is_within_cwd(&cwd, &resolved) {
        return None;
    }
    Some(resolved)
}

pub fn resolve_user_md_request(arg: &str) -> Result<(String, PathBuf), String> {
    if let Some(path) = is_existing_md_file_path(arg) {
        let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        return Ok((text, work_dir_for_path(&path)));
    }
    Ok((arg.trim().to_string(), PathBuf::from(".")))
}

#[cfg(test)]
mod tests {
    use super::{
        looks_like_md_file_path_arg, md_path_has_invalid_characters,
        md_path_has_invalid_components, resolve_user_md_request,
    };
    use std::path::Path;

    #[test]
    fn looks_like_md_file_path_arg_rejects_null_dot_parent_and_space() {
        assert!(looks_like_md_file_path_arg("plan.md"));
        assert!(looks_like_md_file_path_arg("  nested/plan.md  "));
        assert!(!looks_like_md_file_path_arg(""));
        assert!(!looks_like_md_file_path_arg("plan.mdx"));
        assert!(!looks_like_md_file_path_arg("my plan.md"));
        assert!(
            !looks_like_md_file_path_arg("bad\0.md"),
            "NUL must not be treated as a path"
        );
        assert!(!looks_like_md_file_path_arg("../escape.md"));
        assert!(!looks_like_md_file_path_arg("./local.md"));
        assert!(md_path_has_invalid_characters("x\0y"));
        assert!(!md_path_has_invalid_characters("ok.md"));
        assert!(md_path_has_invalid_components(Path::new("../x.md")));
        assert!(md_path_has_invalid_components(Path::new("./x.md")));
        assert!(!md_path_has_invalid_components(Path::new("dir/x.md")));
    }

    #[test]
    fn resolve_user_md_request_rejects_symlink_outside_cwd() {
        #[cfg(unix)]
        {
            use super::is_existing_md_file_path;
            use std::os::unix::fs::symlink;
            let _guard = crate::test_utils::test_env_lock();
            let tmp = tempfile::tempdir().unwrap();
            let outside = tempfile::tempdir().unwrap();
            std::fs::write(outside.path().join("secret.md"), "stolen").unwrap();
            let old_cwd = std::env::current_dir().unwrap();
            std::env::set_current_dir(tmp.path()).unwrap();
            symlink(
                outside.path().join("secret.md"),
                tmp.path().join("steal.md"),
            )
            .unwrap();
            assert!(is_existing_md_file_path("steal.md").is_none());
            let (text, wd) = resolve_user_md_request("steal.md").unwrap();
            assert_eq!(text, "steal.md");
            assert_eq!(wd, std::path::PathBuf::from("."));
            std::env::set_current_dir(old_cwd).unwrap();
        }
    }

    #[test]
    fn resolve_user_md_request_treats_nul_path_as_literal_text() {
        let raw = "bad\0.md";
        let (text, wd) = resolve_user_md_request(raw).expect("literal");
        assert_eq!(text, raw);
        assert_eq!(wd, std::path::PathBuf::from("."));
    }
}
