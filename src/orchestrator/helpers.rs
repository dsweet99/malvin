fn insert_formatted(ctx: &mut HashMap<String, String>, key: &str, path: &Path, base: &Path) {
    ctx.insert(key.to_string(), format_prompt_path(path, base));
}

fn insert_artifact_paths(context: &mut HashMap<String, String>, artifacts: &RunArtifacts) {
    context.insert(
        "plan_path".to_string(),
        artifacts.plan_path.display().to_string(),
    );
    let base = &artifacts.work_dir;
    insert_formatted(context, "grounding_path", &base.join("grounding.md"), base);
    let kpop_dir = artifacts
        .run_dir
        .join("_kpop")
        .canonicalize()
        .unwrap_or_else(|_| artifacts.run_dir.join("_kpop"));
    insert_formatted(context, "kpop_log_dir", &kpop_dir, base);
    insert_formatted(context, "review_path", &artifacts.workspace_review_md(), base);
    insert_formatted(context, "result_path", &artifacts.artifact_result_md(), base);
}

#[must_use]
pub fn workflow_context(
    artifacts: &RunArtifacts,
    prompts: &PromptStore,
) -> HashMap<String, String> {
    let mut context = HashMap::new();
    insert_artifact_paths(&mut context, artifacts);
    match prompts.render_prompt_only("kpop.md", &context) {
        Ok(kpop_content) => {
            context.insert("kpop".to_string(), kpop_content);
        }
        Err(e) => {
            eprintln!("warning: failed to render kpop.md template: {e}");
        }
    }
    context
}

pub(crate) fn clear_review_file(p: &Path) {
    let _ = std::fs::remove_file(p);
}

pub(crate) fn check_abort(result_path: &Path) -> Option<String> {
    std::fs::read_to_string(result_path)
        .ok()
        .filter(|content| content.lines().any(|line| line.starts_with("ABORT:")))
}

/// Stem used in log name segments for **both** coder prompts (`validate_plan.md`, `implement.md`, …) and reviewer prompts (`review_1.md`, …).
/// Strips a trailing `.md` when present (case-sensitive); otherwise returns `filename` unchanged. Avoids panics on short names.
#[must_use]
pub(crate) fn prompt_md_stem(filename: &str) -> &str {
    filename.strip_suffix(".md").unwrap_or(filename)
}

pub(crate) fn format_prompt_path(path: &Path, base_dir: &Path) -> String {
    let path_r = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let base_r = base_dir.canonicalize().unwrap_or_else(|_| base_dir.to_path_buf());
    path_r.strip_prefix(&base_r).map_or_else(
        |_| path.display().to_string(),
        |r| format!("./{}", r.display()),
    )
}

#[cfg(test)]
mod helper_tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn insert_formatted_adds_formatted_path() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("file.md");
        std::fs::write(&path, "").unwrap();
        let mut ctx = HashMap::new();
        insert_formatted(&mut ctx, "key", &path, tmp.path());
        assert!(ctx.get("key").unwrap().contains("file.md"));
    }

    #[test]
    fn insert_artifact_paths_populates_context() {
        let tmp = tempfile::tempdir().unwrap();
        let run_dir = tmp.path().join("_malvin").join("run");
        std::fs::create_dir_all(&run_dir).unwrap();
        let plan_path = run_dir.join("plan.md");
        std::fs::write(&plan_path, "plan").unwrap();
        let artifacts = crate::artifacts::RunArtifacts {
            run_dir,
            plan_path,
            work_dir: tmp.path().to_path_buf(),
        };
        let mut ctx = HashMap::new();
        insert_artifact_paths(&mut ctx, &artifacts);
        assert!(ctx.contains_key("plan_path"));
        assert!(ctx.contains_key("grounding_path"));
        assert!(ctx.contains_key("kpop_log_dir"));
        assert!(ctx.contains_key("review_path"));
        assert!(ctx.contains_key("result_path"));
    }
}

