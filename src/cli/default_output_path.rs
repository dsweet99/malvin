use std::path::{Path, PathBuf};

const DEFAULT_SIBLING_MAX: usize = 9999;

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
        "malvin: failed to allocate a free write output pair for `{}` after {DEFAULT_SIBLING_MAX} attempts",
        tex_default.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_tex_pdf_pair_invents_siblings_when_default_exists() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let tex = tmp.path().join("write.tex");
        let pdf = tmp.path().join("write.pdf");
        std::fs::write(&tex, "x\n").expect("write");
        std::fs::write(&pdf, b"%PDF").expect("write");
        let (got_tex, got_pdf) =
            allocate_default_tex_pdf_pair(&tex, &pdf, "write").expect("alloc");
        assert_eq!(got_tex, tmp.path().join("write_1.tex"));
        assert_eq!(got_pdf, tmp.path().join("write_1.pdf"));
    }
}
