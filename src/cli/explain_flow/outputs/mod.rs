//! Explain product path resolution and validation.

use super::prep::discover_explain_outputs_in_work_dir;
use super::run_startup::ExplainKpopPrepared;

pub(super) fn resolve_explain_output_paths(
    prepared: &ExplainKpopPrepared,
) -> Result<(std::path::PathBuf, std::path::PathBuf), String> {
    if prepared.auto_out_path {
        let discovered = discover_explain_outputs_in_work_dir(
            &prepared.request_work_dir,
            &prepared.preflight_snapshot,
        )?;
        return Ok((discovered.tex_path, discovered.pdf_path));
    }
    Ok((prepared.tex_path.clone(), prepared.pdf_path.clone()))
}

pub(crate) fn validate_explain_output(
    tex_path: &std::path::Path,
    pdf_path: &std::path::Path,
) -> Result<(), String> {
    for (label, path) in [("tex", tex_path), ("pdf", pdf_path)] {
        let meta = std::fs::metadata(path).map_err(|_| {
            format!(
                "malvin explain: expected {label} file at `{}`",
                path.display()
            )
        })?;
        if !meta.is_file() || meta.len() == 0 {
            return Err(format!(
                "malvin explain: expected non-empty {label} file at `{}`",
                path.display()
            ));
        }
    }
    Ok(())
}

pub(super) fn products_nonempty(tex_path: &std::path::Path, pdf_path: &std::path::Path) -> bool {
    validate_explain_output(tex_path, pdf_path).is_ok()
}

#[cfg(test)]
mod outputs_cov;
