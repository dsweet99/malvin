use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::artifacts::resolve_user_md_request;
use crate::cli::cli_request::require_cli_request;
use crate::cli::default_output_path::allocate_default_tex_pdf_pair;
use crate::prompts::{EXPLAIN_WRAPPER_MD, PromptError, PromptStore};

pub(crate) const EXPLAIN_TEX_BASENAME: &str = "explain.tex";
pub(crate) const EXPLAIN_PDF_BASENAME: &str = "explain.pdf";

#[derive(Debug)]
pub(crate) struct ExplainResolvedOutputs {
    pub tex_path: PathBuf,
    pub pdf_path: PathBuf,
}

pub(crate) fn explain_pdf_path_from_tex(tex_path: &Path) -> PathBuf {
    tex_path.with_extension("pdf")
}

fn resolve_explain_output_in_cwd(work_dir: &Path, basename: &str, cwd: &Path) -> PathBuf {
    if work_dir.as_os_str() == "." {
        return cwd.join(basename);
    }
    let rel = work_dir.join(basename);
    if rel.is_absolute() {
        rel
    } else {
        cwd.join(rel)
    }
}

pub(crate) fn explain_resolved_output_paths(
    request_work_dir: &Path,
    out_path: &str,
) -> Result<ExplainResolvedOutputs, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let tex_path = if out_path == EXPLAIN_TEX_BASENAME {
        resolve_explain_output_in_cwd(request_work_dir, EXPLAIN_TEX_BASENAME, &cwd)
    } else {
        let path = Path::new(out_path);
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            cwd.join(path)
        }
    };
    let pdf_path = if out_path == EXPLAIN_TEX_BASENAME {
        resolve_explain_output_in_cwd(request_work_dir, EXPLAIN_PDF_BASENAME, &cwd)
    } else {
        explain_pdf_path_from_tex(&tex_path)
    };
    Ok(ExplainResolvedOutputs {
        tex_path,
        pdf_path,
    })
}

/// Compose the router REQUEST for `malvin explain` (embeds the user request).
pub(crate) fn compose_explain_router_request(
    request_text: &str,
    tex_display: &str,
    pdf_display: &str,
) -> Result<String, String> {
    let ctx = HashMap::from([
        ("tex_display".to_string(), tex_display.to_string()),
        ("pdf_display".to_string(), pdf_display.to_string()),
        ("request_text".to_string(), request_text.to_string()),
    ]);
    PromptStore::default_store()
        .render_prompt_only(EXPLAIN_WRAPPER_MD, &ctx)
        .map_err(|e: PromptError| e.0)
}

pub(crate) fn explain_preflight(
    request: Option<&String>,
    out_path: &str,
    out_path_explicit: bool,
) -> Result<(String, ExplainResolvedOutputs), String> {
    let raw = require_cli_request(request, "explain")?;
    let (text, request_work_dir) = resolve_user_md_request(&raw)?;
    let mut outputs = explain_resolved_output_paths(&request_work_dir, out_path)?;
    if out_path == EXPLAIN_TEX_BASENAME || !out_path_explicit {
        let (tex, pdf) =
            allocate_default_tex_pdf_pair(&outputs.tex_path, &outputs.pdf_path, "explain")?;
        outputs.tex_path = tex;
        outputs.pdf_path = pdf;
    } else {
        for path in [&outputs.tex_path, &outputs.pdf_path] {
            if path.exists() {
                return Err(format!(
                    "malvin explain: `{}` already exists; refusing to overwrite",
                    path.display()
                ));
            }
        }
    }
    for path in [&outputs.tex_path, &outputs.pdf_path] {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
        }
    }
    Ok((text, outputs))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_embeds_user_request_and_out_paths() {
        let body = compose_explain_router_request("how gates exit", "explain.tex", "explain.pdf")
            .expect("compose explain request");
        let expected = "Explain the following topic in a short technical LaTeX paper for an intelligent nonspecialist.\n\
Write LaTeX source to `explain.tex` and compile a PDF to `explain.pdf` (both non-empty).\n\
Prefer plain English; introduce field terms at first use. Back claims with evidence or citation; label hypotheses.\n\
Assume the reader will not read underlying source code; explain the algorithms, mathematics, or design ideas.\n\
Do not overwrite unrelated workspace files.\n\
\n\
User request:\n\
\n\
how gates exit\n";
        assert_eq!(body, expected);
        assert!(body.contains("User request:"));
        assert!(body.contains("how gates exit"));
        assert!(body.contains("`explain.tex`"));
        assert!(body.contains("`explain.pdf`"));
    }

    #[test]
    fn explain_preflight_requires_request() {
        let err = explain_preflight(None, EXPLAIN_TEX_BASENAME, false).unwrap_err();
        assert!(err.contains("explain") && err.contains("REQUEST"));
    }
}
