use std::path::Path;

pub(crate) use crate::workflow_context::insert_formatted;
#[cfg(test)]
pub use crate::workflow_context::workflow_context;
pub use crate::workflow_context::{format_prompt_path, workflow_context_paths_only};

pub fn check_abort(result_path: &Path) -> Result<Option<String>, std::io::Error> {
    let content = match std::fs::read_to_string(result_path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    let text = content.strip_prefix('\u{FEFF}').unwrap_or(&content);
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("ABORT:") {
            return Ok(Some(rest.trim_start().to_string()));
        }
    }
    Ok(None)
}

#[must_use]
pub fn format_exp_log_relative(
    artifacts: &crate::artifacts::RunArtifacts,
    exp_log: &Path,
) -> String {
    crate::workflow_context::format_prompt_path(exp_log, &artifacts.work_dir)
}

#[cfg(test)]
mod helpers_kiss_inline {
    use super::*;

    #[test]
    fn format_exp_log_relative_under_work_dir() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let run_dir = tmp.path().join(".malvin/logs").join("run");
        std::fs::create_dir_all(&run_dir).expect("mkdir");
        let exp_log = run_dir.join("exp.log");
        std::fs::write(&exp_log, "x").expect("write");
        let artifacts = crate::artifacts::RunArtifacts {
            run_dir: run_dir.clone(),
            plan_path: run_dir.join("plan.md"),
            work_dir: tmp.path().to_path_buf(),
        };
        let rel = format_exp_log_relative(&artifacts, &exp_log);
        assert!(rel.contains("exp.log"));
    }

    #[test]
    fn insert_artifact_paths_and_resolve_path_against_base() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let run_dir = tmp.path().join(".malvin/logs").join("run");
        std::fs::create_dir_all(&run_dir).expect("mkdir");
        let plan_path = run_dir.join("plan.md");
        std::fs::write(&plan_path, "p").expect("plan");
        let artifacts = crate::artifacts::RunArtifacts {
            run_dir,
            plan_path: plan_path.clone(),
            work_dir: tmp.path().to_path_buf(),
        };
        let ctx = crate::workflow_context::workflow_context_paths_only(
            &artifacts,
            crate::config::DEFAULT_CLI_MODEL,
            false,
        );
        assert!(ctx.contains_key("quality_gates_log"));
        let _ = format_prompt_path(&plan_path, &artifacts.work_dir);
    }
}
