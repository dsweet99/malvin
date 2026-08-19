use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::artifacts::resolve_user_md_request;
use crate::cli::cli_request::require_cli_request;
use crate::cli::default_output_path::allocate_default_tex_pdf_pair;
use crate::prompts::{PromptError, PromptStore, WRITE_A_MD, WRITE_B_MD};

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
    Ok(WriteResolvedOutputs { tex_path, pdf_path })
}

pub(crate) fn compose_write_a_prompt(
    request_text: &str,
    workspace_dir: &str,
) -> Result<String, String> {
    let ctx = HashMap::from([
        ("workspace_dir".to_string(), workspace_dir.to_string()),
        ("request_text".to_string(), request_text.to_string()),
    ]);
    PromptStore::default_store()
        .render_prompt_only(WRITE_A_MD, &ctx)
        .map_err(|e: PromptError| e.0)
}

pub(crate) fn compose_write_b_prompt(
    tex_display: &str,
    pdf_display: &str,
    workspace_dir: &str,
) -> Result<String, String> {
    let ctx = HashMap::from([
        ("tex_display".to_string(), tex_display.to_string()),
        ("pdf_display".to_string(), pdf_display.to_string()),
        ("workspace_dir".to_string(), workspace_dir.to_string()),
    ]);
    PromptStore::default_store()
        .render_prompt_only(WRITE_B_MD, &ctx)
        .map_err(|e: PromptError| e.0)
}

pub(crate) fn write_preflight(
    request: Option<&String>,
    out_path: &str,
    out_path_explicit: bool,
) -> Result<(String, PathBuf, WriteResolvedOutputs), String> {
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
    Ok((text, request_work_dir, outputs))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_write_a_embeds_request_and_workspace() {
        let body = compose_write_a_prompt("how gates exit", "./.malvin_home/logs/run")
            .expect("compose write_a");
        let expected = crate::prompts::render_template(
            include_str!("../../../default_prompts/write_a.md"),
            &HashMap::from([
                (
                    "workspace_dir".to_string(),
                    "./.malvin_home/logs/run".to_string(),
                ),
                ("request_text".to_string(), "how gates exit".to_string()),
            ]),
        );
        assert_eq!(body, expected);
        assert!(body.contains("how gates exit"));
        assert!(body.contains("notes.tex"));
        assert!(body.contains("./.malvin_home/logs/run"));
    }

    #[test]
    fn compose_write_b_embeds_out_paths_and_workspace() {
        let body = compose_write_b_prompt("write.tex", "write.pdf", "./.malvin_home/logs/run")
            .expect("compose write_b");
        let expected = crate::prompts::render_template(
            include_str!("../../../default_prompts/write_b.md"),
            &HashMap::from([
                ("tex_display".to_string(), "write.tex".to_string()),
                ("pdf_display".to_string(), "write.pdf".to_string()),
                (
                    "workspace_dir".to_string(),
                    "./.malvin_home/logs/run".to_string(),
                ),
            ]),
        );
        assert_eq!(body, expected);
        assert!(body.contains("`write.tex`"));
        assert!(body.contains("`write.pdf`"));
        assert!(body.contains("notes.tex"));
        assert!(body.contains("./.malvin_home/logs/run"));
        assert!(body.contains("Write to the output paths given above while you work"));
        assert!(body.contains("all lowercase, snake_case"));
        assert!(body.contains("At the end, rename both"));
    }

    #[test]
    fn write_preflight_requires_request() {
        let err = write_preflight(None, WRITE_TEX_BASENAME, false).unwrap_err();
        assert!(err.contains("write") && err.contains("REQUEST"));
    }
}
