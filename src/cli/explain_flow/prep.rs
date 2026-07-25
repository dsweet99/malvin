use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::artifacts::resolve_user_md_request;
use crate::prompts::{PromptError, PromptStore};

use super::super::{WorkflowCliOptions, prepare_kpop_prompt_store};
use crate::cli::default_output_path::allocate_default_tex_pdf_pair;
use crate::kpop_program::render_creative_program;

#[path = "prep_discover.rs"]
pub(crate) mod prep_discover;
#[path = "prep_output.rs"]
mod prep_output;

pub(crate) use prep_discover::{
    discover_explain_outputs_in_work_dir, resolve_explain_search_dir, snapshot_tex_pdf_in_dir,
};
pub(crate) use prep_output::{explain_locate_instruction, explain_output_instruction};

pub(crate) const EXPLAIN_TEX_BASENAME: &str = "explain.tex";
pub(crate) const EXPLAIN_PDF_BASENAME: &str = "explain.pdf";

#[derive(Debug)]
pub(crate) struct ExplainResolvedOutputs {
    pub tex_path: PathBuf,
    pub pdf_path: PathBuf,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ExplainPreflightSnapshot {
    pub pre_existing_tex_pdf: std::collections::HashSet<PathBuf>,
}

pub(crate) struct ExplainKpopRequestInput<'a> {
    pub request_text: &'a str,
    pub request_work_dir: &'a Path,
    pub outputs: &'a ExplainResolvedOutputs,
    pub out_path_explicit: bool,
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
    let store = prepare_kpop_prompt_store(workflow, true)?;
    for name in [
        "kpop_program_creative.md",
        "explain_constraints.md",
        "explain_plan.md",
        "explain_work.md",
        "explain_kpop_common.md",
        "explain_kpop_turn.md",
    ] {
        store
            .validate_exists(name)
            .map_err(|e: PromptError| e.0)?;
    }
    Ok(store)
}

const EXPLAIN_REVIEW_JUDGE_PREFACE: &str = "\
Role: explain review.

Judge lack-of-satisfaction of the constraints below. Do **not** edit files. Do **not** write or \
compile LaTeX. Do **not** satisfy constraints by authoring the paper.

Hard rule — no product, no LGTM: if the resolved `.tex` / `.pdf` are missing or empty, emit a \
failure-focused gap list of everything that needs doing. Never return LGTM when products are \
missing or empty.
";

pub(crate) fn explain_kpop_request(
    store: &PromptStore,
    artifacts: &crate::artifacts::RunArtifacts,
    input: ExplainKpopRequestInput<'_>,
) -> Result<String, String> {
    let workspace_root = artifacts.work_dir.as_path();
    let mut ctx = HashMap::new();
    ctx.insert("explain_request".to_string(), input.request_text.to_string());
    ctx.insert(
        "explain_locate_instruction".to_string(),
        explain_locate_instruction(
            input.out_path_explicit,
            input.request_work_dir,
            input.outputs,
            workspace_root,
        ),
    );
    let creative = render_creative_program(store, "explain_constraints.md", &ctx, artifacts)?;
    Ok(format!("{EXPLAIN_REVIEW_JUDGE_PREFACE}\n{creative}"))
}

pub(crate) fn explain_plan_request(store: &PromptStore, review: &str) -> Result<String, String> {
    let mut ctx = HashMap::new();
    ctx.insert("review".to_string(), review.to_string());
    store
        .render_prompt_only("explain_plan.md", &ctx)
        .map_err(|e: PromptError| e.0)
}

pub(crate) struct ExplainWorkPromptParts<'a> {
    pub paths: ExplainKpopRequestInput<'a>,
    pub review: &'a str,
    pub plan: &'a str,
}

pub(crate) fn explain_work_request(
    store: &PromptStore,
    artifacts: &crate::artifacts::RunArtifacts,
    parts: ExplainWorkPromptParts<'_>,
) -> Result<String, String> {
    let workspace_root = artifacts.work_dir.as_path();
    let mut ctx = HashMap::new();
    ctx.insert("review".to_string(), parts.review.to_string());
    ctx.insert("plan".to_string(), parts.plan.to_string());
    ctx.insert(
        "explain_output_instruction".to_string(),
        explain_output_instruction(
            parts.paths.out_path_explicit,
            parts.paths.request_work_dir,
            parts.paths.outputs,
            workspace_root,
        ),
    );
    store
        .render_prompt_only("explain_work.md", &ctx)
        .map_err(|e: PromptError| e.0)
}

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

#[cfg(test)]
#[path = "../explain_flow_prep_tests.rs"]
mod explain_flow_prep_tests;
#[cfg(test)]
#[path = "../explain_flow_prep_preflight_tests.rs"]
mod explain_flow_prep_preflight_tests;
