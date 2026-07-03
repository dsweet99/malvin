//! **`KPopProgram`** — assembles `kpop_program.md` / `kpop_program_creative.md` from soft constraints.

use std::collections::HashMap;
use std::path::Path;

use crate::artifacts::RunArtifacts;
use crate::prompts::{PromptError, PromptStore};

struct RenderRepoProgram<'a> {
    store: &'a PromptStore,
    constraints_prompt: &'a str,
    constraints_context: &'a HashMap<String, String>,
    artifacts: &'a RunArtifacts,
    program_prompt: &'a str,
    include_quality_gates: bool,
}

pub(crate) fn render_repo_program(
    store: &PromptStore,
    constraints_prompt: &str,
    constraints_context: &HashMap<String, String>,
    artifacts: &RunArtifacts,
) -> Result<String, String> {
    render_program_with_template(RenderRepoProgram {
        store,
        constraints_prompt,
        constraints_context,
        artifacts,
        program_prompt: "kpop_program.md",
        include_quality_gates: true,
    })
}

/// Like [`render_repo_program`] but omits reading `.malvin/checks` for `{{ quality_gates }}`.
pub(crate) fn render_repo_program_without_quality_gates(
    store: &PromptStore,
    constraints_prompt: &str,
    constraints_context: &HashMap<String, String>,
    artifacts: &RunArtifacts,
) -> Result<String, String> {
    render_program_with_template(RenderRepoProgram {
        store,
        constraints_prompt,
        constraints_context,
        artifacts,
        program_prompt: "kpop_program.md",
        include_quality_gates: false,
    })
}

pub(crate) fn render_creative_program(
    store: &PromptStore,
    constraints_prompt: &str,
    constraints_context: &HashMap<String, String>,
    artifacts: &RunArtifacts,
) -> Result<String, String> {
    render_program_with_template(RenderRepoProgram {
        store,
        constraints_prompt,
        constraints_context,
        artifacts,
        program_prompt: "kpop_program_creative.md",
        include_quality_gates: false,
    })
}

fn render_program_with_template(input: RenderRepoProgram<'_>) -> Result<String, String> {
    let scope_constraints = input
        .store
        .render_prompt_only(input.constraints_prompt, input.constraints_context)
        .map_err(|e: PromptError| e.0)?;
    let context = if input.include_quality_gates {
        kpop_program_context(
            input.artifacts.work_dir.as_path(),
            &scope_constraints,
            input.artifacts,
        )?
    } else {
        kpop_program_context_without_quality_gates(
            &scope_constraints,
            input.artifacts,
        )
    };
    input
        .store
        .render_prompt_only(input.program_prompt, &context)
        .map(|s| s.trim().to_string())
        .map_err(|e: PromptError| e.0)
}

pub(crate) fn kpop_program_context(
    work_dir: &Path,
    scope_constraints: &str,
    artifacts: &RunArtifacts,
) -> Result<HashMap<String, String>, String> {
    let quality_gates =
        crate::repo_gates::prompt_quality_gates_markdown_ephemeral(work_dir)?;
    Ok(kpop_program_context_with_quality_gates(
        scope_constraints,
        artifacts,
        quality_gates,
    ))
}

fn kpop_program_context_without_quality_gates(
    scope_constraints: &str,
    artifacts: &RunArtifacts,
) -> HashMap<String, String> {
    kpop_program_context_with_quality_gates(scope_constraints, artifacts, String::new())
}

fn kpop_program_context_with_quality_gates(
    scope_constraints: &str,
    artifacts: &RunArtifacts,
    quality_gates: String,
) -> HashMap<String, String> {
    let mut context = HashMap::new();
    context.insert(
        "scope_constraints".to_string(),
        scope_constraints.trim().to_string(),
    );
    context.insert("quality_gates".to_string(), quality_gates);
    context.insert(
        "quality_gates_path".to_string(),
        crate::format_prompt_path(
            &artifacts.quality_gates_log_path(),
            &artifacts.work_dir,
        ),
    );
    context
}

#[cfg(test)]
#[path = "kpop_program_kiss_cov.rs"]
mod kpop_program_kiss_cov;
