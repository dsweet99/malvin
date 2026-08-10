use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::artifacts::resolve_user_md_request;
use crate::cli::cli_request::require_cli_request;
use crate::cli::default_output_path::allocate_default_tex_pdf_pair;
use crate::prompts::{WRITE_WRAPPER_MD, PromptError, PromptStore};

pub(crate) const WRITE_TEX_BASENAME: &str = "write.tex";
pub(crate) const WRITE_PDF_BASENAME: &str = "write.pdf";

#[derive(Debug)]
pub(crate) struct WriteResolvedOutputs {
    pub tex_path: PathBuf,
    pub pdf_path: PathBuf,
}

pub(crate) fn write_pdf_path_from_tex(tex_path: &Path) -> PathBuf {
    tex_path.with_extension("pdf")
}

fn resolve_write_output_in_cwd(work_dir: &Path, basename: &str, cwd: &Path) -> PathBuf {
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

pub(crate) fn write_resolved_output_paths(
    request_work_dir: &Path,
    out_path: &str,
) -> Result<WriteResolvedOutputs, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let tex_path = if out_path == WRITE_TEX_BASENAME {
        resolve_write_output_in_cwd(request_work_dir, WRITE_TEX_BASENAME, &cwd)
    } else {
        let path = Path::new(out_path);
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            cwd.join(path)
        }
    };
    let pdf_path = if out_path == WRITE_TEX_BASENAME {
        resolve_write_output_in_cwd(request_work_dir, WRITE_PDF_BASENAME, &cwd)
    } else {
        write_pdf_path_from_tex(&tex_path)
    };
    Ok(WriteResolvedOutputs {
        tex_path,
        pdf_path,
    })
}

/// Compose the router REQUEST for `malvin write` (embeds the user request).
pub(crate) fn compose_write_router_request(
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
        .render_prompt_only(WRITE_WRAPPER_MD, &ctx)
        .map_err(|e: PromptError| e.0)
}

pub(crate) fn write_preflight(
    request: Option<&String>,
    out_path: &str,
    out_path_explicit: bool,
) -> Result<(String, WriteResolvedOutputs), String> {
    let raw = require_cli_request(request, "write")?;
    let (text, request_work_dir) = resolve_user_md_request(&raw)?;
    let mut outputs = write_resolved_output_paths(&request_work_dir, out_path)?;
    if out_path == WRITE_TEX_BASENAME || !out_path_explicit {
        let (tex, pdf) =
            allocate_default_tex_pdf_pair(&outputs.tex_path, &outputs.pdf_path, "write")?;
        outputs.tex_path = tex;
        outputs.pdf_path = pdf;
    } else {
        for path in [&outputs.tex_path, &outputs.pdf_path] {
            if path.exists() {
                return Err(format!(
                    "malvin write: `{}` already exists; refusing to overwrite",
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
        let body = compose_write_router_request("how gates exit", "write.tex", "write.pdf")
            .expect("compose write request");
        let expected = crate::prompts::render_template(
            include_str!("../../../default_prompts/write_wrapper.md"),
            &HashMap::from([
                ("tex_display".to_string(), "write.tex".to_string()),
                ("pdf_display".to_string(), "write.pdf".to_string()),
                ("request_text".to_string(), "how gates exit".to_string()),
            ]),
        );
        assert_eq!(body, expected);
        assert!(body.contains("User request:"));
        assert!(body.contains("how gates exit"));
        assert!(body.contains("`write.tex`"));
        assert!(body.contains("`write.pdf`"));
    }

    #[test]
    fn write_preflight_requires_request() {
        let err = write_preflight(None, WRITE_TEX_BASENAME, false).unwrap_err();
        assert!(err.contains("write") && err.contains("REQUEST"));
    }
}
