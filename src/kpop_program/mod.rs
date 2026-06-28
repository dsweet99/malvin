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
    })
}

fn render_program_with_template(input: RenderRepoProgram<'_>) -> Result<String, String> {
    let include_quality_gates = input.program_prompt == "kpop_program.md";
    let scope_constraints = input
        .store
        .render_prompt_only(input.constraints_prompt, input.constraints_context)
        .map_err(|e: PromptError| e.0)?;
    let context = if include_quality_gates {
        kpop_program_context(
            input.artifacts.work_dir.as_path(),
            &scope_constraints,
            input.artifacts,
        )?
    } else {
        HashMap::from([(
            "scope_constraints".to_string(),
            scope_constraints.trim().to_string(),
        )])
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
    Ok(context)
}

#[cfg(test)]
#[path = "kpop_program_kiss_cov.rs"]
mod kpop_program_kiss_cov;
