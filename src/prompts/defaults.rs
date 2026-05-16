//! Embedded default prompt bodies (`default_prompts/`).

pub const HEADER_MD: &str = "header.md";
pub const DO_HEADER_MD: &str = "do_header.md";

pub const REQUIRED_PROMPTS: &[&str] = &[
    "check_plan.md",
    "implement.md",
    "review_descriptions.md",
    "reviewer_template.md",
    "review_write.md",
    "concerns.md",
    HEADER_MD,
    "coding_rules.md",
    "tidy_concerns.md",
];

pub const DEFAULT_PROMPTS: &[&str] = &[
    "check_plan.md",
    "implement.md",
    "review_descriptions.md",
    "reviewer_template.md",
    "review_write.md",
    "kpop.md",
    "kpop_common.md",
    "kpop_block.md",
    "mbc2_pure.md",
    "mbc2.md",
    "concerns.md",
    "learn.md",
    "summary.md",
    "tidy.md",
    "tidy_concerns.md",
    "review_plan.md",
    "bug_regression_test.md",
    "bug_fix.md",
    HEADER_MD,
    DO_HEADER_MD,
    "coding_rules.md",
];

pub fn default_file(name: &str) -> Option<&'static str> {
    match name {
        "check_plan.md" => Some(include_str!("../../default_prompts/check_plan.md")),
        "implement.md" => Some(include_str!("../../default_prompts/implement.md")),
        "review_descriptions.md" => {
            Some(include_str!("../../default_prompts/review_descriptions.md"))
        }
        "reviewer_template.md" => Some(include_str!("../../default_prompts/reviewer_template.md")),
        "review_write.md" => Some(include_str!("../../default_prompts/review_write.md")),
        "kpop.md" | "kpop_common.md" => Some(include_str!("../../default_prompts/kpop_common.md")),
        "kpop_block.md" => Some(include_str!("../../default_prompts/kpop_block.md")),
        "mbc2_pure.md" => Some(include_str!("../../default_prompts/mbc2_pure.md")),
        "mbc2.md" => Some(include_str!("../../default_prompts/mbc2.md")),
        "concerns.md" => Some(include_str!("../../default_prompts/concerns.md")),
        "learn.md" => Some(include_str!("../../default_prompts/learn.md")),
        "summary.md" => Some(include_str!("../../default_prompts/summary.md")),
        "tidy.md" => Some(include_str!("../../default_prompts/tidy.md")),
        "tidy_concerns.md" => Some(include_str!("../../default_prompts/tidy_concerns.md")),
        "review_plan.md" => Some(include_str!("../../default_prompts/review_plan.md")),
        "bug_regression_test.md" => {
            Some(include_str!("../../default_prompts/bug_regression_test.md"))
        }
        "bug_fix.md" => Some(include_str!("../../default_prompts/bug_fix.md")),
        HEADER_MD => Some(include_str!("../../default_prompts/header.md")),
        DO_HEADER_MD => Some(include_str!("../../default_prompts/do_header.md")),
        "coding_rules.md" => Some(include_str!("../../default_prompts/coding_rules.md")),
        _ => None,
    }
}

#[cfg(test)]
mod review_plan_embed_tests {
    use super::default_file;

    #[test]
    fn embedded_review_plan_starts_with_kpop_placeholder_line() {
        let s = default_file("review_plan.md").expect("review_plan must be embedded");
        assert!(s.contains("{{ kpop }}"));
    }
}

#[cfg(test)]
mod review_write_embed_tests {
    use super::default_file;

    #[test]
    fn embedded_review_write_writes_problems_only_to_review_path() {
        let s = default_file("review_write.md").expect("review_write must be embedded");
        assert!(
            s.contains("{{ review_path }}"),
            "review_write must target review_path"
        );
        assert!(
            s.to_ascii_lowercase().contains("regression test"),
            "review_write must ask for regression tests for remaining bug findings"
        );
    }
}

#[cfg(test)]
mod reviewer_template_embed_tests {
    use super::default_file;

    #[test]
    fn embedded_reviewer_template_spells_uncommitted_correctly() {
        let s = default_file("reviewer_template.md").expect("reviewer_template must be embedded");
        assert!(
            !s.contains("uncommited"),
            "embedded reviewer_template must not contain the common misspelling"
        );
    }
}

#[cfg(test)]
mod review_descriptions_embed_tests {
    use super::default_file;

    #[test]
    fn embedded_review_descriptions_has_no_stray_close_paren_question() {
        let s = default_file("review_descriptions.md")
            .expect("review_descriptions must be embedded");
        assert!(
            !s.contains("metrics)?"),
            "kiss-metrics line must not contain stray ')?' fragment"
        );
    }

    #[test]
    fn review_descriptions_is_required_for_code_workflow() {
        assert!(
            super::REQUIRED_PROMPTS.contains(&"review_descriptions.md"),
            "malvin code fan-out loads review_descriptions.md at runtime; validate_required must list it"
        );
    }
}

#[cfg(test)]
mod do_header_tests {
    use super::DO_HEADER_MD;
    use super::default_file;

    #[test]
    fn embedded_do_header_is_a_single_text_block_with_closing_newline() {
        let s = default_file(DO_HEADER_MD).expect("do header must be embedded");
        let lower = s.to_ascii_lowercase();
        assert!(s.ends_with('\n'));
        assert!(lower.contains("no stream of consciousness"));
        assert!(!s.contains("You'll\n find"));
    }
}
