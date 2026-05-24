// Embedded default prompt bodies (`default_prompts/`).

pub const REVIEW_WRITE_ACP_MATCH_PHRASE: &str = "write your final review";

pub const CONCERNS_ACP_MATCH_SUBSTRING: &str = "reviewer's concerns";

pub const HEADER_MD: &str = "header.md";
pub const DO_HEADER_MD: &str = "do_header.md";

pub const REQUIRED_PROMPTS: &[&str] = &[
    "check_plan.md",
    "implement.md",
    "reviewers_spawn.md",
    "review_write.md",
    "concerns.md",
    HEADER_MD,
    "coding_rules.md",
    "tidy_concerns.md",
];

pub const DEFAULT_PROMPTS: &[&str] = &[
    "check_plan.md",
    "implement.md",
    "reviewers_spawn.md",
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
    "hunt_request.md",
    HEADER_MD,
    DO_HEADER_MD,
    "coding_rules.md",
];

pub fn default_file(name: &str) -> Option<&'static str> {
    match name {
        "check_plan.md" => Some(include_str!("../../default_prompts/check_plan.md")),
        "implement.md" => Some(include_str!("../../default_prompts/implement.md")),
        "reviewers_spawn.md" => Some(include_str!("../../default_prompts/reviewers_spawn.md")),
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
        "hunt_request.md" => Some(include_str!("../../default_prompts/hunt_request.md")),
        HEADER_MD => Some(include_str!("../../default_prompts/header.md")),
        DO_HEADER_MD => Some(include_str!("../../default_prompts/do_header.md")),
        "coding_rules.md" => Some(include_str!("../../default_prompts/coding_rules.md")),
        _ => None,
    }
}

#[cfg(test)]
mod review_plan_embed_tests {
    use std::path::Path;

    use super::{DEFAULT_PROMPTS, default_file};
    use crate::artifacts::create_run_artifacts;
    use crate::orchestrator::workflow_context;
    use crate::prompts::{PromptStore, malformed_brace_placeholders};

    #[test]
    fn embedded_default_prompts_use_spaced_brace_placeholders() {
        for name in DEFAULT_PROMPTS {
            let text = default_file(name).unwrap_or_else(|| panic!("{name} must be embedded"));
            let bad = malformed_brace_placeholders(text);
            assert!(bad.is_empty(), "{name}: {bad:?}");
        }
    }

    #[test]
    fn embedded_review_plan_renders_without_unresolved_braces() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let plan_path = tmp.path().join("plan.md");
        std::fs::write(&plan_path, "plan body\n").expect("write plan");
        let artifacts =
            create_run_artifacts(Path::new(&plan_path), Some(tmp.path())).expect("artifacts");
        let store = PromptStore::default_store();
        let mut ctx = workflow_context(&artifacts, &store, "plan").expect("ctx");
        ctx.insert(
            "plan_path".to_string(),
            crate::orchestrator::format_prompt_path(&plan_path, &artifacts.work_dir),
        );
        let out = store.render("review_plan.md", &ctx).expect("render");
        assert!(
            !out.contains("{{"),
            "embedded review_plan.md must expand all placeholders"
        );
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
            s.to_ascii_lowercase()
                .contains(super::REVIEW_WRITE_ACP_MATCH_PHRASE),
            "review_write must include phrase used by ACP integration mocks"
        );
        assert!(
            s.to_ascii_lowercase().contains("regression test"),
            "review_write must ask for regression tests for remaining bug findings"
        );
    }
}

#[cfg(test)]
mod concerns_embed_tests {
    use super::{CONCERNS_ACP_MATCH_SUBSTRING, default_file};

    #[test]
    fn embedded_concerns_prompts_contain_acp_mock_routing_substring() {
        for name in ["concerns.md", "tidy_concerns.md"] {
            let s = default_file(name).unwrap_or_else(|| panic!("{name} must be embedded"));
            assert!(
                s.contains(CONCERNS_ACP_MATCH_SUBSTRING),
                "acp mocks branch on {CONCERNS_ACP_MATCH_SUBSTRING:?} but {name} does not; \
                 see tests/common/acp_core.rs and acp_code_fanout_mocks.rs"
            );
        }
    }
}

#[cfg(test)]
mod reviewers_spawn_embed_tests {
    use super::default_file;

    #[test]
    fn reviewers_spawn_is_required_for_code_workflow() {
        assert!(
            super::REQUIRED_PROMPTS.contains(&"reviewers_spawn.md"),
            "malvin code review loads reviewers_spawn.md at runtime; validate_required must list it"
        );
    }

    #[test]
    fn embedded_reviewers_spawn_writes_prep_and_uses_kpop() {
        let s = default_file("reviewers_spawn.md").expect("reviewers_spawn must be embedded");
        assert!(
            s.contains("{{ review_prep_path }}"),
            "reviewers_spawn must write review_prep_path"
        );
        assert!(
            s.contains("{{ kpop }}"),
            "reviewers_spawn must include kpop block"
        );
        assert!(
            s.contains("KPop: Review in-scope code for these problems"),
            "reviewers_spawn must run the single review pass"
        );
        assert!(
            !s.contains("Spawn one subagent for each of these prompts"),
            "reviewers_spawn must not coordinate subagent fan-out"
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
