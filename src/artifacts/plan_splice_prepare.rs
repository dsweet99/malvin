//! Pre-Prompt-1a machine-block shell for `malvin plan`.

use std::path::Path;

use super::{
    append_machine_block, ensure_user_span_trailing_newlines, plan_splice_boundary,
    read_plan_file, write_plan_file_atomic, PlanFileError,
};

pub(crate) const RESTATEMENT_SECTION_STUB: &str = "## Restatement\n";

/// Atomically append the canonical `\n---\nBEGIN_MALVIN\n## Restatement\n` shell before Prompt 1a.
pub fn prepare_plan_file_for_prompt_1a(path: &Path) -> Result<(), PlanFileError> {
    let content = read_plan_file(path)?;
    if plan_splice_boundary::count_begin_malvin_marker_lines(&content) > 0 {
        return Ok(());
    }
    let mut spliced = content;
    ensure_user_span_trailing_newlines(&mut spliced);
    append_machine_block(&mut spliced, RESTATEMENT_SECTION_STUB);
    write_plan_file_atomic(path, &spliced)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifacts::{detect_rerun_user_span_end, validate_post_1a};

    #[test]
    fn kiss_cov_prepare_prompt_1a() {
        let _ = prepare_plan_file_for_prompt_1a;
        let _ = RESTATEMENT_SECTION_STUB;
    }

    #[test]
    fn prepare_plan_file_for_prompt_1a_appends_machine_shell() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("plan.md");
        std::fs::write(&path, "# User plan\n\nDo the thing.\n").expect("write");
        prepare_plan_file_for_prompt_1a(&path).expect("prep");
        let out = std::fs::read_to_string(&path).expect("read");
        assert!(out.ends_with("---\nBEGIN_MALVIN\n## Restatement\n"));
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
        assert!(out.starts_with(user));
        assert!(out.ends_with("---\nBEGIN_MALVIN\n## Restatement\n"));
        let user_span_end = detect_rerun_user_span_end(&out).expect("detect").expect("span");
        assert_eq!(&out[..user_span_end], user);
    }

    #[test]
    fn prepare_plan_file_for_prompt_1a_is_idempotent_when_shell_present() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("plan.md");
        let seeded = "# User\n\n---\nBEGIN_MALVIN\n## Restatement\n";
        std::fs::write(&path, seeded).expect("write");
        prepare_plan_file_for_prompt_1a(&path).expect("prep");
        assert_eq!(std::fs::read_to_string(&path).expect("read"), seeded);
    }
}
