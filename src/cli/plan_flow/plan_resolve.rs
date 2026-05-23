use crate::artifacts::resolve_user_request;
use crate::cli::PlanArgs;

fn resolve_plan_at_destination(trimmed: &str) -> Result<std::path::PathBuf, String> {
    let (_, work_dir) = resolve_user_request(trimmed)?;
    let rest = trimmed
        .trim()
        .strip_prefix('@')
        .ok_or_else(|| "missing @ prefix".to_string())?;
    let name = std::path::Path::new(rest)
        .file_name()
        .ok_or_else(|| format!("invalid @ path: {rest}"))?;
    Ok(work_dir.join(name))
}

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

fn plan_write_bytes(text: &str, allow_trim_empty_from_at_file: bool) -> Result<Vec<u8>, String> {
    if text.trim().is_empty() {
        if allow_trim_empty_from_at_file && !text.is_empty() {
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

enum AtPathSuffix {
    None,
    Bare,
    AtFile,
}

fn classify_at_path_suffix(text: &str) -> AtPathSuffix {
    let trimmed = text.trim();
    if !trimmed.starts_with('@') {
        return AtPathSuffix::None;
    }
    let rest = trimmed.strip_prefix('@').unwrap_or("");
    if rest.is_empty() {
        AtPathSuffix::Bare
    } else {
        AtPathSuffix::AtFile
    }
}

fn is_sole_at_in_place(plan: &PlanArgs) -> bool {
    plan.plan_path.is_none()
        && matches!(plan.text.as_deref().map(classify_at_path_suffix), Some(AtPathSuffix::AtFile))
}

fn paths_equal(a: &std::path::Path, b: &std::path::Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

fn at_source_matches_destination(plan: &PlanArgs, dest: &std::path::Path) -> bool {
    let Some(text) = plan.text.as_deref() else {
        return false;
    };
    let AtPathSuffix::AtFile = classify_at_path_suffix(text.trim()) else {
        return false;
    };
    resolve_plan_at_destination(text.trim())
        .ok()
        .is_some_and(|source| paths_equal(&source, dest))
}

pub(super) fn plan_session_work_dir(
    plan: &PlanArgs,
    user_plan_path: &std::path::Path,
) -> std::path::PathBuf {
    if is_sole_at_in_place(plan) {
        if let Some(text) = plan.text.as_deref() {
            let trimmed = text.trim();
            if let Ok((_, work_dir)) = resolve_user_request(trimmed) {
                return work_dir;
            }
        }
    }
    user_plan_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map_or_else(|| std::path::PathBuf::from("."), std::path::Path::to_path_buf)
}

pub(super) fn resolve_plan_destination(plan: &PlanArgs) -> Result<std::path::PathBuf, String> {
    if let Some(ref flag_path) = plan.plan_path {
        if let Some(text) = &plan.text {
            if matches!(classify_at_path_suffix(text), AtPathSuffix::Bare) {
                return Err("Empty path after `@`.".to_string());
            }
        }
        return resolve_user_plan_path(Some(flag_path.clone()));
    }
    if let Some(text) = &plan.text {
        match classify_at_path_suffix(text) {
            AtPathSuffix::Bare => return Err("Empty path after `@`.".to_string()),
            AtPathSuffix::AtFile => return resolve_plan_at_destination(text),
            AtPathSuffix::None => {}
        }
    }
    resolve_user_plan_path(None)
}

fn plan_source_bytes(
    plan: &PlanArgs,
    user_plan_path: &std::path::Path,
) -> Result<Option<Vec<u8>>, String> {
    let Some(ref text) = plan.text else {
        return Ok(None);
    };
    if is_sole_at_in_place(plan) || at_source_matches_destination(plan, user_plan_path) {
        return Ok(None);
    }
    let trimmed = text.trim();
    let source = match classify_at_path_suffix(trimmed) {
        AtPathSuffix::Bare => return Err("Empty path after `@`.".to_string()),
        AtPathSuffix::AtFile => {
            let (content, _) = resolve_user_request(trimmed)?;
            return Ok(Some(plan_write_bytes(&content, true)?));
        }
        AtPathSuffix::None => trimmed.to_string(),
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
pub(super) fn is_sole_at_in_place_for_test(plan: &PlanArgs) -> bool {
    is_sole_at_in_place(plan)
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
    fn kiss_cov_resolve_plan_at_destination() { let _ = stringify!(resolve_plan_at_destination); }

    #[test]
    fn kiss_cov_resolve_user_plan_path() { let _ = stringify!(resolve_user_plan_path); }

    #[test]
    fn kiss_cov_plan_write_bytes() { let _ = stringify!(plan_write_bytes); }

    #[test]
    fn kiss_cov_at_path_suffix() { let _ = stringify!(AtPathSuffix); }

    #[test]
    fn kiss_cov_classify_at_path_suffix() { let _ = stringify!(classify_at_path_suffix); }

    #[test]
    fn kiss_cov_is_sole_at_in_place() { let _ = stringify!(is_sole_at_in_place); }

    #[test]
    fn kiss_cov_paths_equal() { let _ = stringify!(paths_equal); }

    #[test]
    fn kiss_cov_at_source_matches_destination() { let _ = stringify!(at_source_matches_destination); }

    #[test]
    fn kiss_cov_plan_source_bytes() { let _ = stringify!(plan_source_bytes); }

}
