use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::workflow_context::insert_formatted;

use super::ExplainResolvedOutputs;
use super::prep_discover::resolve_explain_search_dir;

const AUTO_OUTPUT_INSTRUCTION: &str = "- Write LaTeX source and compile a matching PDF in `{{ explain_work_dir }}`. Give the output file a name that is a shortened version of the report's title. Use all lowercase and snake case.";

const AUTO_LOCATE_INSTRUCTION: &str = "- Locate existing explanation products in `{{ explain_work_dir }}` (LaTeX `.tex` and matching `.pdf`, often a shortened snake_case title). Do not write or compile files in this review.";

pub(crate) fn explain_output_instruction(
    out_path_explicit: bool,
    request_work_dir: &Path,
    outputs: &ExplainResolvedOutputs,
    workspace_root: &Path,
) -> String {
    if out_path_explicit {
        return explain_explicit_output_instruction(outputs, workspace_root);
    }
    explain_auto_output_instruction(request_work_dir, workspace_root)
}

pub(crate) fn explain_locate_instruction(
    out_path_explicit: bool,
    request_work_dir: &Path,
    outputs: &ExplainResolvedOutputs,
    workspace_root: &Path,
) -> String {
    if out_path_explicit {
        return explain_explicit_locate_instruction(outputs, workspace_root);
    }
    explain_auto_locate_instruction(request_work_dir, workspace_root)
}

fn explain_explicit_output_instruction(
    outputs: &ExplainResolvedOutputs,
    workspace_root: &Path,
) -> String {
    let mut ctx = HashMap::new();
    insert_formatted(&mut ctx, "explain_tex_path", &outputs.tex_path, workspace_root);
    insert_formatted(&mut ctx, "explain_pdf_path", &outputs.pdf_path, workspace_root);
    let tex = ctx.get("explain_tex_path").cloned().unwrap_or_default();
    let pdf = ctx.get("explain_pdf_path").cloned().unwrap_or_default();
    format!("- Write LaTeX source to `{tex}` and compile it to `{pdf}`.")
}

fn explain_explicit_locate_instruction(
    outputs: &ExplainResolvedOutputs,
    workspace_root: &Path,
) -> String {
    let mut ctx = HashMap::new();
    insert_formatted(&mut ctx, "explain_tex_path", &outputs.tex_path, workspace_root);
    insert_formatted(&mut ctx, "explain_pdf_path", &outputs.pdf_path, workspace_root);
    let tex = ctx.get("explain_tex_path").cloned().unwrap_or_default();
    let pdf = ctx.get("explain_pdf_path").cloned().unwrap_or_default();
    format!(
        "- Locate existing explanation products at `{tex}` and `{pdf}` if present. Do not write or compile files in this review."
    )
}

fn explain_work_dir_formatted(request_work_dir: &Path, workspace_root: &Path) -> String {
    let mut ctx = HashMap::new();
    insert_formatted(
        &mut ctx,
        "explain_work_dir",
        &resolve_explain_search_dir(
            request_work_dir,
            &std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        ),
        workspace_root,
    );
    ctx.get("explain_work_dir").cloned().unwrap_or_default()
}

fn explain_auto_output_instruction(request_work_dir: &Path, workspace_root: &Path) -> String {
    let work_dir = explain_work_dir_formatted(request_work_dir, workspace_root);
    AUTO_OUTPUT_INSTRUCTION.replace("{{ explain_work_dir }}", &work_dir)
}

fn explain_auto_locate_instruction(request_work_dir: &Path, workspace_root: &Path) -> String {
    let work_dir = explain_work_dir_formatted(request_work_dir, workspace_root);
    AUTO_LOCATE_INSTRUCTION.replace("{{ explain_work_dir }}", &work_dir)
}
