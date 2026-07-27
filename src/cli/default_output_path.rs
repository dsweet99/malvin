use std::path::{Path, PathBuf};

const DEFAULT_SIBLING_MAX: usize = 9999;

pub(crate) const DELIGHT_DEFAULT_OUT_PATH: &str = "pitch.md";

/// Relative path string for `path` when it lies under the process cwd.
pub(crate) fn path_relative_to_cwd(path: &Path) -> Result<String, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    if let Ok(rel) = path.strip_prefix(&cwd) {
        let s = rel.to_string_lossy();
        if s.is_empty() {
            return Err("malvin: empty output path relative to cwd".into());
        }
        return Ok(s.into_owned());
    }
    Ok(path.to_string_lossy().into_owned())
}

/// When `default_path` is occupied, return the first free `{stem}_{n}{extension}` sibling.
pub(crate) fn allocate_default_sibling_file(
    default_path: &Path,
    stem: &str,
    extension: &str,
) -> Result<PathBuf, String> {
    if !default_path.exists() {
        return Ok(default_path.to_path_buf());
    }
    let parent = default_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    for n in 1..=DEFAULT_SIBLING_MAX {
        let candidate = parent.join(format!("{stem}_{n}{extension}"));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(format!(
        "malvin: failed to allocate a free sibling name for `{}` after {DEFAULT_SIBLING_MAX} attempts",
        default_path.display()
    ))
}

/// When the default tex/pdf pair is occupied, allocate matching `{stem}_{n}.tex` / `.pdf` siblings.
pub(crate) fn allocate_default_tex_pdf_pair(
    tex_default: &Path,
    pdf_default: &Path,
    stem: &str,
) -> Result<(PathBuf, PathBuf), String> {
    if !tex_default.exists() && !pdf_default.exists() {
        return Ok((tex_default.to_path_buf(), pdf_default.to_path_buf()));
    }
    let parent = tex_default
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    for n in 1..=DEFAULT_SIBLING_MAX {
        let tex = parent.join(format!("{stem}_{n}.tex"));
        let pdf = parent.join(format!("{stem}_{n}.pdf"));
        if !tex.exists() && !pdf.exists() {
            return Ok((tex, pdf));
        }
    }
    Err(format!(
        "malvin: failed to allocate a free explain output pair for `{}` after {DEFAULT_SIBLING_MAX} attempts",
        tex_default.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_sibling_returns_default_when_free() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let default = tmp.path().join("plan.md");
        let got = allocate_default_sibling_file(&default, "plan", ".md").expect("alloc");
        assert_eq!(got, default);
    }

    #[test]
    fn allocate_sibling_invents_plan_1_when_default_exists() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let default = tmp.path().join("plan.md");
        std::fs::write(&default, "x\n").expect("write");
        let got = allocate_default_sibling_file(&default, "plan", ".md").expect("alloc");
        assert_eq!(got, tmp.path().join("plan_1.md"));
    }

    #[test]
    fn allocate_tex_pdf_pair_invents_siblings_when_default_exists() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let tex = tmp.path().join("explain.tex");
        let pdf = tmp.path().join("explain.pdf");
        std::fs::write(&tex, "x\n").expect("write");
        std::fs::write(&pdf, b"%PDF").expect("write");
        let (got_tex, got_pdf) =
            allocate_default_tex_pdf_pair(&tex, &pdf, "explain").expect("alloc");
        assert_eq!(got_tex, tmp.path().join("explain_1.tex"));
        assert_eq!(got_pdf, tmp.path().join("explain_1.pdf"));
    }
}
