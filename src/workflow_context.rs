use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::artifacts::RunArtifacts;
use crate::memory_context;

use crate::prompts::{PromptError, PromptStore};

pub(crate) fn insert_formatted(ctx: &mut HashMap<String, String>, key: &str, path: &Path, base: &Path) {
    ctx.insert(key.to_string(), format_prompt_path(path, base));
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
    insert_formatted(context, "malvin_output_path", &artifacts.run_dir, base);
    insert_formatted(
        context,
        "quality_gates_log",
        &artifacts.quality_gates_log_path(),
        base,
    );
}

#[must_use]
pub fn workflow_context_paths_only(
    artifacts: &RunArtifacts,
    malvin_command: &str,
) -> HashMap<String, String> {
    let mut context = HashMap::new();
    insert_artifact_paths(&mut context, artifacts);
    context.insert(
        "memories".to_string(),
        memory_context::build_memories_value(&artifacts.work_dir),
    );
    context.insert("malvin_command".to_string(), malvin_command.to_string());
    context
}

/// Builds the full workflow render context (paths, memories, quality gates, `kpop` slot).
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
mod workflow_context_path_tests {
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

    use super::{
        format_prompt_path, insert_artifact_paths, insert_formatted, resolve_nonexistent_path,
        resolve_path_against_base,
    };

    #[test]
    fn resolve_path_against_base_resolves_relative_plan_path() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let base = tmp.path().canonicalize().expect("base");
        let resolved = resolve_path_against_base(Path::new("plan.md"), &base);
        assert!(resolved.ends_with("plan.md"));
    }

    #[test]
    fn resolve_path_against_base_resolves_absolute_missing_file_under_base() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let base = tmp.path().canonicalize().expect("base");
        let abs = tmp.path().join("src/foo.rs");
        let resolved = resolve_path_against_base(&abs, &base);
        assert!(
            resolved.starts_with(&base),
            "expected resolved path under base, got {}",
            resolved.display()
        );
        assert!(resolved.ends_with("src/foo.rs"));
    }

    #[test]
    fn resolve_nonexistent_path_cases() {
        let _ = stringify!(resolve_nonexistent_path);
        assert_eq!(resolve_nonexistent_path(Path::new("")), PathBuf::from(""));

        let tmp = tempfile::tempdir().expect("tempdir");
        let base = tmp.path().canonicalize().expect("base");
        let missing = tmp.path().join("nested/missing.md");
        let resolved = resolve_nonexistent_path(&missing);
        assert!(resolved.starts_with(&base));
        assert!(resolved.ends_with("nested/missing.md"));

        let deep = tmp.path().join("a/b/c/d.md");
        let deep_resolved = resolve_nonexistent_path(&deep);
        assert!(deep_resolved.starts_with(&base));
        assert!(deep_resolved.ends_with("a/b/c/d.md"));
    }

    #[test]
    fn format_prompt_path_fallback_uses_resolved_path_display() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let outside = std::env::temp_dir().join(format!("malvin_outside_{}", std::process::id()));
        std::fs::create_dir_all(&outside).expect("outside dir");
        let missing = outside.join("missing.md");
        let formatted = format_prompt_path(&missing, tmp.path());
        assert!(
            !formatted.starts_with("./"),
            "outside path must not be relativized: {formatted}"
        );
        assert!(
            formatted.contains("missing.md"),
            "fallback must name the file: {formatted}"
        );
        let _ = std::fs::remove_dir_all(&outside);
    }

    #[test]
    fn insert_artifact_paths_populates_expected_keys() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let plan = tmp.path().join("plan.md");
        std::fs::write(&plan, "p").expect("write");
        let artifacts = crate::artifacts::create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
        let mut ctx = HashMap::new();
        insert_artifact_paths(&mut ctx, &artifacts);
        assert!(ctx.contains_key("result_path"));
        assert!(ctx.contains_key("review_prep_path"));
    }

    #[test]
    fn insert_formatted_stores_workflow_relative_path() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let plan = tmp.path().join("plan.md");
        std::fs::write(&plan, "p").expect("write");
        let mut ctx = HashMap::new();
        insert_formatted(&mut ctx, "plan_path", &plan, tmp.path());
        assert_eq!(ctx.get("plan_path").map(String::as_str), Some("./plan.md"));
    }
}
