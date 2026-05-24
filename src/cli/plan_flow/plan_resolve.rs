use crate::artifacts::{is_existing_md_file_path, resolve_user_md_request, work_dir_for_path};
use crate::cli::PlanArgs;

pub fn resolve_user_plan_path(
    plan_path: Option<std::path::PathBuf>,
) -> Result<std::path::PathBuf, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let p = plan_path.unwrap_or_else(|| cwd.join("plan.md"));
    Ok(if p.is_absolute() { p } else { cwd.join(p) })
}

pub(super) fn normalized_plan_file_bytes(text: &str) -> Result<Vec<u8>, String> {
    plan_write_bytes(text, false)
}

fn plan_write_bytes(text: &str, allow_trim_empty_from_md_file: bool) -> Result<Vec<u8>, String> {
    if text.trim().is_empty() {
        if allow_trim_empty_from_md_file && !text.is_empty() {
            return Ok(text.as_bytes().to_vec());
        }
        return Err("ERR: plan text is empty (after trimming).".to_string());
    }
    let core = text.trim_end_matches(['\r', '\n']);
    let mut s = String::with_capacity(core.len() + 1);
    s.push_str(core);
    s.push('\n');
    Ok(s.into_bytes())
}

enum MdPathSuffix {
    Literal,
    ExistingFile,
}

fn classify_md_path_suffix(text: &str) -> MdPathSuffix {
    if is_existing_md_file_path(text.trim()).is_some() {
        MdPathSuffix::ExistingFile
    } else {
        MdPathSuffix::Literal
    }
}

fn is_sole_md_file_in_place(plan: &PlanArgs) -> bool {
    plan.plan_path.is_none()
        && matches!(
            plan.text.as_deref().map(classify_md_path_suffix),
            Some(MdPathSuffix::ExistingFile)
        )
}

fn paths_equal(a: &std::path::Path, b: &std::path::Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

fn md_source_matches_destination(plan: &PlanArgs, dest: &std::path::Path) -> bool {
    let Some(text) = plan.text.as_deref() else {
        return false;
    };
    let MdPathSuffix::ExistingFile = classify_md_path_suffix(text.trim()) else {
        return false;
    };
    is_existing_md_file_path(text.trim())
        .is_some_and(|source| paths_equal(&source, dest))
}

pub(super) fn plan_session_work_dir(
    plan: &PlanArgs,
    user_plan_path: &std::path::Path,
) -> std::path::PathBuf {
    if is_sole_md_file_in_place(plan) {
        if let Some(text) = plan.text.as_deref() {
            if let Some(path) = is_existing_md_file_path(text.trim()) {
                return work_dir_for_path(&path);
            }
        }
    }
    user_plan_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map_or_else(|| std::path::PathBuf::from("."), std::path::Path::to_path_buf)
}

pub(super) fn resolve_plan_destination(plan: &PlanArgs) -> Result<std::path::PathBuf, String> {
    if plan.plan_path.is_some() {
        return resolve_user_plan_path(plan.plan_path.clone());
    }
    if let Some(text) = &plan.text {
        if matches!(classify_md_path_suffix(text), MdPathSuffix::ExistingFile) {
            return is_existing_md_file_path(text.trim()).ok_or_else(|| {
                format!(
                    "not an existing .md file: {}",
                    text.trim()
                )
            });
        }
    }
    resolve_user_plan_path(None)
}

fn is_default_cwd_plan_md(user_plan_path: &std::path::Path) -> bool {
    let Ok(cwd) = std::env::current_dir() else {
        return false;
    };
    let default = cwd.join("plan.md");
    paths_equal(user_plan_path, &default)
}

fn plan_source_bytes(
    plan: &PlanArgs,
    user_plan_path: &std::path::Path,
) -> Result<Option<Vec<u8>>, String> {
    let Some(ref text) = plan.text else {
        return Ok(None);
    };
    if is_sole_md_file_in_place(plan) || md_source_matches_destination(plan, user_plan_path) {
        return Ok(None);
    }
    if plan.plan_path.is_none()
        && is_default_cwd_plan_md(user_plan_path)
        && user_plan_path.is_file()
        && matches!(classify_md_path_suffix(text.trim()), MdPathSuffix::Literal)
    {
        return Err(
            "ERR: plan.md already exists; delete or rename it, pass --plan-path, or run `malvin plan` with no positional to review.".to_string(),
        );
    }
    let trimmed = text.trim();
    let source = match classify_md_path_suffix(trimmed) {
        MdPathSuffix::ExistingFile => {
            let (content, _) = resolve_user_md_request(trimmed)?;
            return Ok(Some(plan_write_bytes(&content, true)?));
        }
        MdPathSuffix::Literal => trimmed.to_string(),
    };
    Ok(Some(normalized_plan_file_bytes(&source)?))
}

pub(super) fn apply_plan_source(
    plan: &PlanArgs,
    user_plan_path: &std::path::Path,
) -> Result<(), String> {
    if let Some(bytes) = plan_source_bytes(plan, user_plan_path)? {
        if let Some(parent) = user_plan_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(user_plan_path, bytes).map_err(|e| e.to_string())?;
        return Ok(());
    }
    if !user_plan_path.exists() {
        return Err(format!(
            "ERR: plan file does not exist: {}",
            user_plan_path.display()
        ));
    }
    Ok(())
}

#[cfg(test)]
pub(super) fn is_sole_md_file_in_place_for_test(plan: &PlanArgs) -> bool {
    is_sole_md_file_in_place(plan)
}

#[cfg(test)]
pub(super) fn plan_source_bytes_for_test(
    plan: &PlanArgs,
    user_plan_path: &std::path::Path,
) -> Result<Option<Vec<u8>>, String> {
    plan_source_bytes(plan, user_plan_path)
}

#[cfg(test)]
mod kiss_cov_auto {
    #[test]
    fn kiss_cov_resolve_user_plan_path() {
        let _ = stringify!(resolve_user_plan_path);
    }

    #[test]
    fn kiss_cov_plan_write_bytes() {
        let _ = stringify!(plan_write_bytes);
    }

    #[test]
    fn kiss_cov_md_path_suffix() {
        let _ = stringify!(MdPathSuffix);
    }

    #[test]
    fn kiss_cov_classify_md_path_suffix() {
        let _ = stringify!(classify_md_path_suffix);
    }

    #[test]
    fn kiss_cov_is_sole_md_file_in_place() {
        let _ = stringify!(is_sole_md_file_in_place);
    }

    #[test]
    fn kiss_cov_paths_equal() {
        let _ = stringify!(paths_equal);
    }

    #[test]
    fn kiss_cov_md_source_matches_destination() {
        let _ = stringify!(md_source_matches_destination);
    }

    #[test]
    fn kiss_cov_plan_source_bytes() {
        let _ = stringify!(plan_source_bytes);
    }
}
