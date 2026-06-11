use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::artifacts::resolve_user_md_request;
use crate::prompts::{PromptError, PromptStore};
use crate::workflow_context::insert_formatted;

use super::super::{WorkflowCliOptions, prepare_kpop_prompt_store};
use crate::cli::workflow_kpop_shared::render_kpop_program_request_creative;

pub(crate) const EXPLAIN_TEX_BASENAME: &str = "explain.tex";
pub(crate) const EXPLAIN_PDF_BASENAME: &str = "explain.pdf";

#[derive(Debug)]
pub(crate) struct ExplainResolvedOutputs {
    pub tex_path: PathBuf,
    pub pdf_path: PathBuf,
}

fn resolve_explain_output_in_cwd(work_dir: &Path, basename: &str, cwd: &Path) -> PathBuf {
    let rel = work_dir.join(basename);
    if rel.is_absolute() {
        rel
    } else {
        cwd.join(rel)
    }
}

pub(crate) fn explain_pdf_path_from_tex(tex_path: &Path) -> PathBuf {
    tex_path.with_extension("pdf")
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

#[cfg(test)]
pub(crate) fn explain_output_paths(work_dir: &Path) -> ExplainResolvedOutputs {
    explain_resolved_output_paths(work_dir, EXPLAIN_TEX_BASENAME)
        .expect("default explain output paths must resolve")
}

pub(crate) fn prepare_explain_kpop_prompt_store(
    workflow: WorkflowCliOptions,
) -> Result<PromptStore, String> {
    let store = prepare_kpop_prompt_store(workflow, false)?;
    store
        .validate_exists("kpop_program_creative.md")
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("explain_constraints.md")
        .map_err(|e: PromptError| e.0)?;
    Ok(store)
}

pub(crate) fn explain_kpop_request(
    store: &PromptStore,
    artifacts: &crate::artifacts::RunArtifacts,
    request_text: &str,
    outputs: &ExplainResolvedOutputs,
) -> Result<String, String> {
    let workspace_root = artifacts.work_dir.as_path();
    let mut ctx = HashMap::new();
    ctx.insert("explain_request".to_string(), request_text.to_string());
    insert_formatted(&mut ctx, "explain_tex_path", &outputs.tex_path, workspace_root);
    insert_formatted(&mut ctx, "explain_pdf_path", &outputs.pdf_path, workspace_root);
    render_kpop_program_request_creative(store, "explain_constraints.md", &ctx, artifacts)
}

pub(crate) fn explain_revise_doc_path(request: &str, out_path: &str) -> Result<String, String> {
    let (_, request_work_dir) = resolve_user_md_request(request)?;
    let outputs = explain_resolved_output_paths(&request_work_dir, out_path)?;
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    if let Ok(rel) = outputs.tex_path.strip_prefix(&cwd) {
        let rel_str = rel.to_string_lossy();
        if rel_str.is_empty() {
            return Err("malvin explain: empty revise doc path".into());
        }
        return Ok(rel_str.into_owned());
    }
    Ok(outputs.tex_path.to_string_lossy().into_owned())
}

pub(crate) fn explain_preflight(
    request: &str,
    out_path: &str,
) -> Result<(String, ExplainResolvedOutputs), String> {
    let (text, request_work_dir) = resolve_user_md_request(request)?;
    let outputs = explain_resolved_output_paths(&request_work_dir, out_path)?;
    for path in [&outputs.tex_path, &outputs.pdf_path] {
        if path.exists() {
            return Err(format!(
                "malvin explain: `{}` already exists; refusing to overwrite",
                path.display()
            ));
        }
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
        }
    }
    Ok((text, outputs))
}

#[cfg(test)]
#[path = "../explain_flow_prep_tests.rs"]
mod explain_flow_prep_tests;

#[cfg(test)]
#[path = "../explain_flow_prep_preflight_tests.rs"]
mod explain_flow_prep_preflight_tests;
