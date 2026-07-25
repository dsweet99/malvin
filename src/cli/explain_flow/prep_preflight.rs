use std::path::PathBuf;

use crate::artifacts::resolve_user_md_request;
use crate::cli::default_output_path::allocate_default_tex_pdf_pair;

use super::prep_discover::{resolve_explain_search_dir, snapshot_tex_pdf_in_dir};
use super::{
    explain_resolved_output_paths, ExplainPreflightSnapshot, ExplainResolvedOutputs,
    EXPLAIN_PDF_BASENAME, EXPLAIN_TEX_BASENAME,
};

fn explain_auto_preflight(
    text: String,
    request_work_dir: PathBuf,
) -> Result<(String, PathBuf, ExplainResolvedOutputs, ExplainPreflightSnapshot), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let search_dir = resolve_explain_search_dir(&request_work_dir, &cwd);
    let snapshot = ExplainPreflightSnapshot {
        pre_existing_tex_pdf: if search_dir.is_dir() {
            snapshot_tex_pdf_in_dir(&search_dir)?
        } else {
            std::collections::HashSet::default()
        },
    };
    let outputs = ExplainResolvedOutputs {
        tex_path: search_dir.join(EXPLAIN_TEX_BASENAME),
        pdf_path: search_dir.join(EXPLAIN_PDF_BASENAME),
    };
    Ok((text, request_work_dir, outputs, snapshot))
}

fn explain_explicit_preflight(
    text: String,
    request_work_dir: PathBuf,
    out_path: &str,
) -> Result<(String, PathBuf, ExplainResolvedOutputs, ExplainPreflightSnapshot), String> {
    let mut outputs = explain_resolved_output_paths(&request_work_dir, out_path)?;
    if out_path == EXPLAIN_TEX_BASENAME {
        let (tex, pdf) = allocate_default_tex_pdf_pair(
            &outputs.tex_path,
            &outputs.pdf_path,
            "explain",
        )?;
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
    Ok((text, request_work_dir, outputs, ExplainPreflightSnapshot::default()))
}

pub(crate) fn explain_preflight(
    request: &str,
    out_path: &str,
    out_path_explicit: bool,
) -> Result<(String, PathBuf, ExplainResolvedOutputs, ExplainPreflightSnapshot), String> {
    let (text, request_work_dir) = resolve_user_md_request(request)?;
    if out_path_explicit {
        explain_explicit_preflight(text, request_work_dir, out_path)
    } else {
        explain_auto_preflight(text, request_work_dir)
    }
}
