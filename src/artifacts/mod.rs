mod create;
mod md_request;
mod startup_tag;

use std::path::{Path, PathBuf};

pub use create::{
    create_run_artifacts, create_run_artifacts_from_text, create_run_artifacts_from_text_opts,
    create_run_artifacts_opts,
};
pub(crate) use create::{ensure_gate_exp_log_file, ensure_quality_gates_log_file};

pub use crate::session_dotfile_backup::{
    GitignoreBackup, MalvinChecksBackup, MalvinConfigWorkspaceBackup, SessionDotfileBackups,
    SessionDotfileParts, VisionBackup, backup_workspace_gitignore_if_present,
    backup_workspace_gitignore_if_present_with_id, backup_workspace_malvin_checks_if_present,
    backup_workspace_malvin_checks_if_present_with_id,
    backup_workspace_malvin_config_workspace_if_present,
    backup_workspace_malvin_config_workspace_if_present_with_id,
    backup_workspace_vision_if_present, backup_workspace_vision_if_present_with_id,
    merge_and_sanitize_for_gate_restore, merge_for_gate_restore,
    repair_invalid_malvin_home_config_on_disk, restore_workspace_gitignore_backup,
    restore_workspace_malvin_checks_backup, restore_workspace_malvin_config_workspace_backup,
    restore_workspace_session_dotfiles, restore_workspace_vision_backup,
};

pub use md_request::{
    is_existing_md_file_path, looks_like_md_file_path_arg, resolve_user_md_request,
};
pub use startup_tag::startup_request_tag_label;

pub use crate::malvin_constants::{QUALITY_GATES_LOG, SANDBOX_OOM_JSON, STDOUT_LOG, TRACE_JSONL};

#[derive(Debug, Clone)]
pub struct RunArtifacts {
    pub run_dir: PathBuf,
    pub plan_path: PathBuf,
    pub work_dir: PathBuf,
}

impl RunArtifacts {
    #[must_use]
    pub fn log_path(&self, name: &str) -> PathBuf {
        let safe = name.replace(['/', '\\'], "_");
        self.run_dir.join(format!("{safe}.log"))
    }

    #[must_use]
    pub fn artifact_review_md(&self) -> PathBuf {
        self.run_dir.join("review.md")
    }

    #[must_use]
    pub fn review_prep_md(&self) -> PathBuf {
        self.run_dir.join("review_prep.md")
    }

    #[must_use]
    pub fn artifact_result_md(&self) -> PathBuf {
        self.run_dir.join("result.md")
    }

    #[must_use]
    pub fn exp_log_path(&self) -> PathBuf {
        self.gate_exp_log_path(0)
    }

    #[must_use]
    pub fn gate_exp_log_path(&self, iteration: usize) -> PathBuf {
        let slug = self
            .run_dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("run");
        let name = if iteration == 0 {
            format!("exp_log_{slug}.md")
        } else {
            format!("exp_log_{slug}_g{iteration}.md")
        };
        self.run_dir.join("_run").join(name)
    }

    #[must_use]
    pub fn quality_gates_log_path(&self) -> PathBuf {
        self.run_dir.join(QUALITY_GATES_LOG)
    }

    #[must_use]
    pub fn stdout_log_path(&self) -> PathBuf {
        self.run_dir.join(STDOUT_LOG)
    }

    #[must_use]
    pub fn sandbox_oom_json_path(&self) -> PathBuf {
        self.run_dir.join(SANDBOX_OOM_JSON)
    }
}

#[must_use]
pub fn user_request_path(artifacts: &RunArtifacts) -> PathBuf {
    artifacts.run_dir.join("user_request.md")
}

#[must_use]
pub fn review_requirements_json(artifacts: &RunArtifacts) -> PathBuf {
    artifacts.run_dir.join("review_requirements.json")
}

pub(crate) fn work_dir_for_path(path: &Path) -> PathBuf {
    path.parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map_or_else(
            || PathBuf::from("."),
            |parent| {
                parent
                    .canonicalize()
                    .unwrap_or_else(|_| parent.to_path_buf())
            },
        )
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "log_gc_hook_tests.rs"]
mod log_gc_hook_tests;

#[cfg(test)]
#[path = "run_meta_path_tests.rs"]
mod run_meta_path_tests;
