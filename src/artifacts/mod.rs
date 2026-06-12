//! Run directories and log paths.

mod md_request;
mod plan_splice;
mod startup_tag;
mod create;

use std::path::{Path, PathBuf};

pub use create::{
    create_kpop_run_artifacts, create_kpop_run_artifacts_opts, create_run_artifacts,
    create_run_artifacts_from_text, create_run_artifacts_from_text_opts, create_run_artifacts_opts,
};
pub(crate) use create::{ensure_gate_exp_log_file, ensure_quality_gates_log_file};

pub use plan_splice::{
    BEGIN_MALVIN_MARKER, PlanFileError, PlanRunMetadata, detect_rerun_user_span_end,
    extract_decisions_section, extract_fenced_markdown_block, find_machine_block_start,
    is_interrupted_machine_plan, prepare_plan_file_for_prompt_1a, prepare_plan_file_for_run,
    read_plan_file, read_plan_metadata, overwrite_plan_file, plan_user_sidecar_path,
    remove_plan_user_sidecar, restore_interrupted_plan, snapshot_plan_artifact, validate_post_1a,
    validate_post_1b, validate_post_2, write_plan_file_atomic, write_plan_metadata,
};

pub use crate::session_dotfile_backup::{
    GitignoreBackup, KissConfigBackup, KissignoreBackup, MalvinChecksBackup, MalvinConfigBackup,
    MalvinConfigWorkspaceBackup,
    SessionDotfileBackups, SessionDotfileParts, backup_workspace_gitignore_if_present,
    backup_workspace_gitignore_if_present_with_id, backup_workspace_kissconfig_if_present,
    backup_workspace_kissconfig_if_present_with_id, backup_workspace_kissignore_if_present,
    backup_workspace_kissignore_if_present_with_id, backup_workspace_malvin_checks_if_present,
    backup_workspace_malvin_checks_if_present_with_id,
    backup_workspace_malvin_config_if_present, backup_workspace_malvin_config_if_present_with_id,
    backup_workspace_malvin_config_workspace_if_present,
    backup_workspace_malvin_config_workspace_if_present_with_id,
    restore_workspace_gitignore_backup, restore_workspace_kissconfig_backup,
    restore_workspace_kissignore_backup, restore_workspace_malvin_checks_backup,
    restore_workspace_malvin_config_backup, restore_workspace_malvin_config_workspace_backup,
    restore_workspace_session_dotfiles, merge_and_sanitize_for_gate_restore,
    merge_for_gate_restore, repair_clamp_damaged_dotfiles_on_disk,
    sanitize_clamp_damaged_dotfiles_in_bundle,
};

pub use md_request::{is_existing_md_file_path, resolve_user_md_request};
pub use startup_tag::startup_request_tag_label;

pub use crate::malvin_constants::{QUALITY_GATES_LOG, SANDBOX_OOM_JSON, STDOUT_LOG, TRACE_JSONL};

/// One workflow run: isolated `.malvin/logs/<stamp>_<token>/` with copied plan.
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

    /// Run-directory copy of `review.md` (artifact for [`crate::review_sync`]).
    #[must_use]
    pub fn artifact_review_md(&self) -> PathBuf {
        self.run_dir.join("review.md")
    }

    #[must_use]
    pub fn review_prep_md(&self) -> PathBuf {
        self.run_dir.join("review_prep.md")
    }

    /// Run-directory `result.md` for concerns ABORT signaling.
    #[must_use]
    pub fn artifact_result_md(&self) -> PathBuf {
        self.run_dir.join("result.md")
    }

    #[must_use]
    pub fn exp_log_path(&self) -> PathBuf {
        self.gate_exp_log_path(0)
    }

    /// Gate-loop experiment log; `iteration` 0 is the legacy `exp_log_{slug}.md` scaffold.
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
        self.run_dir.join("_kpop").join(name)
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
    pub fn trace_jsonl_path(&self) -> PathBuf {
        self.run_dir.join(TRACE_JSONL)
    }

    #[must_use]
    pub fn sandbox_oom_json_path(&self) -> PathBuf {
        self.run_dir.join(SANDBOX_OOM_JSON)
    }
}

pub(crate) fn work_dir_for_path(path: &Path) -> PathBuf {
    path.parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map_or_else(
            || PathBuf::from("."),
            |parent| parent.canonicalize().unwrap_or_else(|_| parent.to_path_buf()),
        )
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "log_gc_hook_tests.rs"]
mod log_gc_hook_tests;

#[cfg(test)]
#[path = "kpop_path_tests.rs"]
mod kpop_path_tests;
