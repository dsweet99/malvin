use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::artifacts::RunArtifacts;

use crate::prompts::{PromptError, PromptStore};

pub(crate) fn insert_formatted(ctx: &mut HashMap<String, String>, key: &str, path: &Path, base: &Path) {
    ctx.insert(key.to_string(), format_prompt_path(path, base));
}

fn insert_quality_gates_log_paths(
    context: &mut HashMap<String, String>,
    artifacts: &RunArtifacts,
    base: &Path,
) {
    let path = format_prompt_path(&artifacts.quality_gates_log_path(), base);
    context.insert("quality_gates_log".to_string(), path.clone());
    context.insert("quality_gates_path".to_string(), path);
}

fn insert_artifact_paths(context: &mut HashMap<String, String>, artifacts: &RunArtifacts) {
    let base = &artifacts.work_dir;
    insert_formatted(context, "plan_path", &artifacts.plan_path, base);
    let kpop_dir = artifacts
        .run_dir
        .join("_kpop")
        .canonicalize()
        .unwrap_or_else(|_| artifacts.run_dir.join("_kpop"));
    insert_formatted(context, "kpop_log_dir", &kpop_dir, base);
    insert_formatted(
        context,
        "review_path",
        &artifacts.artifact_review_md(),
        base,
    );
    insert_formatted(
        context,
        "review_prep_path",
        &artifacts.review_prep_md(),
        base,
    );
    insert_formatted(
        context,
        "result_path",
        &artifacts.artifact_result_md(),
        base,
    );
    insert_formatted(context, "exp_log", &artifacts.exp_log_path(), base);
    insert_formatted(
        context,
        "advice_path",
        &crate::malvin_advice_path(base),
        base,
    );
    insert_formatted(context, "malvin_output_path", &artifacts.run_dir, base);
    insert_quality_gates_log_paths(context, artifacts, base);
}

fn insert_current_state(
    context: &mut HashMap<String, String>,
    artifacts: &RunArtifacts,
    base: &Path,
) {
    context.insert(
        "current_state".to_string(),
        crate::current_state::format_current_state(base, None, Some(artifacts)),
    );
}

#[must_use]
pub fn workflow_context_paths_only(
    artifacts: &RunArtifacts,
    malvin_command: &str,
) -> HashMap<String, String> {
    let mut context = HashMap::new();
    insert_artifact_paths(&mut context, artifacts);
    insert_current_state(&mut context, artifacts, &artifacts.work_dir);
    context.insert("malvin_command".to_string(), malvin_command.to_string());
    context
}

/// Builds the full workflow render context (paths, quality gates, `kpop` slot).
///
/// # Errors
///
/// Returns [`PromptError`] when quality gate markdown or `kpop_common.md` rendering fails.
pub fn workflow_context(
    artifacts: &RunArtifacts,
    prompts: &PromptStore,
    malvin_command: &str,
) -> Result<HashMap<String, String>, PromptError> {
    let mut context = workflow_context_paths_only(artifacts, malvin_command);
    context.insert(
        "quality_gates".to_string(),
        crate::repo_gates::prompt_quality_gates_markdown_ephemeral(&artifacts.work_dir)
            .map_err(PromptError)?,
    );
    let kpop_content = prompts.render_prompt_only("kpop_common.md", &context)?;
    context.insert("kpop".to_string(), kpop_content);
    Ok(context)
}

fn resolve_path_against_base(path: &Path, base_r: &Path) -> PathBuf {
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        base_r.join(path)
    };
    abs.canonicalize().unwrap_or_else(|_| resolve_nonexistent_path(&abs))
}

fn resolve_nonexistent_path(abs: &Path) -> PathBuf {
    abs.ancestors()
        .find_map(|ancestor| {
            ancestor.canonicalize().ok().map(|canonical| {
                match abs.strip_prefix(ancestor) {
                    Ok(tail) if !tail.as_os_str().is_empty() => canonical.join(tail),
                    _ => canonical,
                }
            })
        })
        .unwrap_or_else(|| abs.to_path_buf())
}

#[must_use]
pub fn format_prompt_path(path: &Path, base_dir: &Path) -> String {
    let base_r = base_dir
        .canonicalize()
        .unwrap_or_else(|_| base_dir.to_path_buf());
    let path_r = resolve_path_against_base(path, &base_r);
    path_r.strip_prefix(&base_r).map_or_else(
        |_| path_r.display().to_string(),
        |r| format!("./{}", r.display()),
    )
}

#[cfg(test)]
#[path = "workflow_context_tests.rs"]
mod workflow_context_tests;
