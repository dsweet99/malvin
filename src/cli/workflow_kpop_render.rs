use std::collections::HashMap;

use crate::artifacts::RunArtifacts;
use crate::prompts::{PromptError, PromptStore};

use super::kpop_program_context;

struct RenderKpopProgram<'a> {
    store: &'a PromptStore,
    constraints_prompt: &'a str,
    constraints_context: &'a HashMap<String, String>,
    artifacts: &'a RunArtifacts,
    program_prompt: &'a str,
}

pub(crate) fn render_kpop_program_request(
    store: &PromptStore,
    constraints_prompt: &str,
    constraints_context: &HashMap<String, String>,
    artifacts: &RunArtifacts,
) -> Result<String, String> {
    render_kpop_program_request_with_template(RenderKpopProgram {
        store,
        constraints_prompt,
        constraints_context,
        artifacts,
        program_prompt: "kpop_program.md",
    })
}

pub(crate) fn render_kpop_program_request_creative(
    store: &PromptStore,
    constraints_prompt: &str,
    constraints_context: &HashMap<String, String>,
    artifacts: &RunArtifacts,
) -> Result<String, String> {
    render_kpop_program_request_with_template(RenderKpopProgram {
        store,
        constraints_prompt,
        constraints_context,
        artifacts,
        program_prompt: "kpop_program_creative.md",
    })
}

fn render_kpop_program_request_with_template(input: RenderKpopProgram<'_>) -> Result<String, String> {
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

#[cfg(test)]
#[path = "workflow_kpop_render_kiss_cov.rs"]
mod workflow_kpop_render_kiss_cov;
