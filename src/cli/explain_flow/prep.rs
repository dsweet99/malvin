use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::artifacts::resolve_user_md_request;
use crate::prompts::{PromptError, PromptStore};
use crate::workflow_context::insert_formatted;

use super::super::{WorkflowCliOptions, prepare_kpop_prompt_store};
use crate::cli::workflow_kpop_shared::render_kpop_program_request;

pub(crate) const EXPLAIN_TEX_BASENAME: &str = "explain.tex";
pub(crate) const EXPLAIN_PDF_BASENAME: &str = "explain.pdf";

pub(crate) fn explain_output_paths(work_dir: &Path) -> (PathBuf, PathBuf) {
    (
        work_dir.join(EXPLAIN_TEX_BASENAME),
        work_dir.join(EXPLAIN_PDF_BASENAME),
    )
}

pub(crate) fn prepare_explain_kpop_prompt_store(
    workflow: WorkflowCliOptions,
) -> Result<PromptStore, String> {
    let store = prepare_kpop_prompt_store(workflow, false)?;
    store
        .validate_exists("kpop_program.md")
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
    work_dir: &Path,
) -> Result<String, String> {
    let (tex_path, pdf_path) = explain_output_paths(work_dir);
    let workspace_root = artifacts.work_dir.as_path();
    let mut ctx = HashMap::new();
    ctx.insert("explain_request".to_string(), request_text.to_string());
    insert_formatted(&mut ctx, "explain_tex_path", &tex_path, workspace_root);
    insert_formatted(&mut ctx, "explain_pdf_path", &pdf_path, workspace_root);
    render_kpop_program_request(store, "explain_constraints.md", &ctx, artifacts)
}

fn resolve_explain_output_in_cwd(work_dir: &Path, basename: &str, cwd: &Path) -> PathBuf {
    let rel = work_dir.join(basename);
    if rel.is_absolute() {
        rel
    } else {
        cwd.join(rel)
    }
}

pub(crate) fn explain_preflight(request: &str) -> Result<(String, PathBuf), String> {
    let (text, work_dir) = resolve_user_md_request(request)?;
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    for basename in [EXPLAIN_TEX_BASENAME, EXPLAIN_PDF_BASENAME] {
        let resolved = resolve_explain_output_in_cwd(&work_dir, basename, &cwd);
        if resolved.exists() {
            return Err(format!(
                "malvin explain: `{}` already exists; refusing to overwrite",
                resolved.display()
            ));
        }
    }
    Ok((text, work_dir))
}

#[cfg(test)]
#[path = "../explain_flow_prep_tests.rs"]
mod explain_flow_prep_tests;

#[cfg(test)]
#[path = "../explain_flow_prep_preflight_tests.rs"]
mod explain_flow_prep_preflight_tests;
