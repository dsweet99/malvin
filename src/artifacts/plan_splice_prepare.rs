//! Pre-Prompt-1a overwrite shell for `malvin plan`.

use std::path::Path;

use super::{
    is_interrupted_machine_plan, overwrite_plan_file, plan_user_sidecar_path, read_plan_file,
    write_plan_file_atomic, PlanFileError,
};

pub(crate) const RESTATEMENT_SECTION_STUB: &str = "## Restatement\n";

/// Atomically overwrite `PLAN_PATH` with `## Restatement` before Prompt 1a; user plan is preserved in a sidecar.
pub fn prepare_plan_file_for_prompt_1a(path: &Path) -> Result<(), PlanFileError> {
    let content = read_plan_file(path)?;
    if is_interrupted_machine_plan(&content) {
        return Ok(());
    }
    let sidecar = plan_user_sidecar_path(path);
    write_plan_file_atomic(&sidecar, &content)?;
    overwrite_plan_file(path, RESTATEMENT_SECTION_STUB)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifacts::validate_post_1a;

    #[test]
    fn kiss_cov_prepare_prompt_1a() {
        let _ = prepare_plan_file_for_prompt_1a;
        let _ = RESTATEMENT_SECTION_STUB;
    }

    #[test]
    fn prepare_plan_file_for_prompt_1a_overwrites_with_restatement_stub() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("plan.md");
        std::fs::write(&path, "# User plan\n\nDo the thing.\n").expect("write");
        prepare_plan_file_for_prompt_1a(&path).expect("prep");
        let out = std::fs::read_to_string(&path).expect("read");
        assert_eq!(out, "## Restatement\n");
        let sidecar = std::fs::read_to_string(plan_user_sidecar_path(&path)).expect("sidecar");
        assert_eq!(sidecar, "# User plan\n\nDo the thing.\n");
        validate_post_1a(&format!("{out}restated\n")).expect("valid after agent restatement");
    }

    #[test]
    fn prepare_plan_file_for_prompt_1a_tolerates_interior_section_dividers() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("plan.md");
        let user = "# Plan\n\n---\n\nSection A\n\n---\n\nSection B\n\n";
        std::fs::write(&path, user).expect("write");
        prepare_plan_file_for_prompt_1a(&path).expect("prep");
        let out = std::fs::read_to_string(&path).expect("read");
        assert_eq!(out, "## Restatement\n");
        assert_eq!(
            std::fs::read_to_string(plan_user_sidecar_path(&path)).expect("sidecar"),
            user
        );
    }

    #[test]
    fn prepare_plan_file_for_prompt_1a_is_idempotent_when_staging_present() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("plan.md");
        let seeded = "## Restatement\nrestated\n";
        std::fs::write(&path, seeded).expect("write");
        prepare_plan_file_for_prompt_1a(&path).expect("prep");
        assert_eq!(std::fs::read_to_string(&path).expect("read"), seeded);
    }
}
