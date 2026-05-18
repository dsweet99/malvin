use std::path::PathBuf;

use crate::artifacts::RunArtifacts;

use super::constants::REVIEW_WRITE_FILE;
use super::prompt_md_stem;

#[derive(Clone, Copy)]
pub struct ReviewPromptLog {
    pub prompt_file: &'static str,
    pub skip_repo_style: bool,
    pub log_attempt: usize,
    pub attempt: usize,
}

fn review_write_inner_try_needs_suffix(input: ReviewPromptLog) -> bool {
    (input.prompt_file == REVIEW_WRITE_FILE || input.attempt != input.log_attempt)
        && input.attempt > 1
}

pub fn review_prompt_log_path(artifacts: &RunArtifacts, input: ReviewPromptLog) -> PathBuf {
    let stem = prompt_md_stem(input.prompt_file);
    let base = format!("{stem}_attempt_{}", input.log_attempt);
    let file_name = if review_write_inner_try_needs_suffix(input) {
        format!("{base}_try_{}", input.attempt)
    } else {
        base
    };
    if input.skip_repo_style {
        return artifacts.log_path(&file_name);
    }
    artifacts.log_path(&format!("coder_{stem}_{file_name}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifacts::create_run_artifacts_from_text;

    #[test]
    fn review_write_inner_try_needs_suffix_for_retry() {
        let input = ReviewPromptLog {
            prompt_file: REVIEW_WRITE_FILE,
            skip_repo_style: false,
            log_attempt: 1,
            attempt: 2,
        };
        assert!(review_write_inner_try_needs_suffix(input));
    }

    fn log_file_name(input: ReviewPromptLog) -> String {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifacts = create_run_artifacts_from_text("review_prompt_log_path", Some(tmp.path()))
            .expect("artifacts");
        review_prompt_log_path(&artifacts, input)
            .file_name()
            .expect("file name")
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn tidy_skip_repo_style_first_review_write_try_uses_outer_attempt_only() {
        assert_eq!(
            log_file_name(ReviewPromptLog {
                prompt_file: "review_write.md",
                skip_repo_style: true,
                log_attempt: 3,
                attempt: 1,
            }),
            "review_write_attempt_3.log"
        );
    }

    #[test]
    fn tidy_skip_repo_style_inner_review_write_retry_adds_try_suffix() {
        assert_eq!(
            log_file_name(ReviewPromptLog {
                prompt_file: "review_write.md",
                skip_repo_style: true,
                log_attempt: 3,
                attempt: 2,
            }),
            "review_write_attempt_3_try_2.log"
        );
    }

    #[test]
    fn tidy_skip_repo_style_inner_review_write_retry_distinct_when_log_attempt_equals_outer_attempt()
     {
        let first_try = log_file_name(ReviewPromptLog {
            prompt_file: "review_write.md",
            skip_repo_style: true,
            log_attempt: 2,
            attempt: 1,
        });
        let second_try = log_file_name(ReviewPromptLog {
            prompt_file: "review_write.md",
            skip_repo_style: true,
            log_attempt: 2,
            attempt: 2,
        });
        assert_eq!(first_try, "review_write_attempt_2.log");
        assert_eq!(
            second_try, "review_write_attempt_2_try_2.log",
            "inner try 2 must not reuse the same log path as try 1 when outer attempt is 2 \
             (got first={first_try:?} second={second_try:?})"
        );
    }

    #[test]
    fn tidy_skip_repo_style_spawn_keeps_outer_attempt_without_try_suffix() {
        assert_eq!(
            log_file_name(ReviewPromptLog {
                prompt_file: "reviewers_spawn.md",
                skip_repo_style: true,
                log_attempt: 4,
                attempt: 4,
            }),
            "reviewers_spawn_attempt_4.log"
        );
    }

    #[test]
    fn code_review_style_review_write_log_distinct_per_outer_loop() {
        assert_eq!(
            log_file_name(ReviewPromptLog {
                prompt_file: "review_write.md",
                skip_repo_style: false,
                log_attempt: 1,
                attempt: 1,
            }),
            "coder_review_write_review_write_attempt_1.log"
        );
        assert_eq!(
            log_file_name(ReviewPromptLog {
                prompt_file: "review_write.md",
                skip_repo_style: false,
                log_attempt: 2,
                attempt: 1,
            }),
            "coder_review_write_review_write_attempt_2.log"
        );
    }

    #[test]
    fn code_review_style_inner_review_write_retry_adds_try_suffix() {
        assert_eq!(
            log_file_name(ReviewPromptLog {
                prompt_file: "review_write.md",
                skip_repo_style: false,
                log_attempt: 3,
                attempt: 2,
            }),
            "coder_review_write_review_write_attempt_3_try_2.log"
        );
    }
}
