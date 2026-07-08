use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::artifacts::resolve_user_md_request;
use crate::prompts::{PromptError, PromptStore};
use crate::workflow_context::insert_formatted;

use super::super::{WorkflowCliOptions, prepare_kpop_prompt_store};
use crate::cli::default_output_path::{
    allocate_default_sibling_file, path_relative_to_cwd, PRIORS_DEFAULT_OUT_PATH,
};
use crate::kpop_program::render_creative_program;

pub(crate) fn prepare_priors_kpop_prompt_store(
    workflow: WorkflowCliOptions,
) -> Result<PromptStore, String> {
    let store = prepare_kpop_prompt_store(workflow, true)?;
    store
        .validate_exists("kpop_program_creative.md")
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("priors_constraints.md")
        .map_err(|e: PromptError| e.0)?;
    Ok(store)
}

pub(crate) fn priors_kpop_request(
    store: &PromptStore,
    artifacts: &crate::artifacts::RunArtifacts,
    resolved_out_path: &Path,
    user_request_disk: &Path,
) -> Result<String, String> {
    let workspace_root = artifacts.work_dir.as_path();
    let mut ctx = HashMap::new();
    insert_formatted(
        &mut ctx,
        "user_request_path",
        user_request_disk,
        workspace_root,
    );
    insert_formatted(&mut ctx, "out_priors_path", resolved_out_path, workspace_root);
    render_creative_program(store, "priors_constraints.md", &ctx, artifacts)
}

pub(crate) fn priors_preflight(
    request: &str,
    out_path: &str,
) -> Result<(String, PathBuf, PathBuf), String> {
    let (request_text, request_work_dir) = resolve_user_md_request(request)?;
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let resolved_out_path = if out_path == PRIORS_DEFAULT_OUT_PATH {
        let default = cwd.join(PRIORS_DEFAULT_OUT_PATH);
        allocate_default_sibling_file(&default, "priors", ".md")?
    } else {
        let resolved = cwd.join(out_path);
        if resolved.exists() {
            return Err(format!(
                "malvin priors: `{}` already exists; refusing to overwrite",
                resolved.display()
            ));
        }
        resolved
    };
    if let Some(parent) = resolved_out_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }
    let rel_out = path_relative_to_cwd(&resolved_out_path)?;
    let out_work_dir = crate::artifacts::work_dir_for_path(Path::new(&rel_out));
    let work_dir = if out_path == PRIORS_DEFAULT_OUT_PATH {
        request_work_dir
    } else {
        out_work_dir
    };
    Ok((request_text, resolved_out_path, work_dir))
}

#[cfg(test)]
#[path = "../priors_flow_prep_tests.rs"]
mod priors_flow_prep_tests;

#[cfg(test)]
#[path = "../priors_flow_prep_preflight_tests.rs"]
mod priors_flow_prep_preflight_tests;
