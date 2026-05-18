//! Bugs from `review_prep.md` § Bugs — untracked `include!` test shards.

use super::helpers::{assert_tracked_in_git, manifest_root};

#[test]
fn kpop_multiturn_prompts_tests_inc_must_be_tracked_when_include_wired() {
    let src = std::fs::read_to_string(manifest_root().join("src/kpop_multiturn_prompts.rs"))
        .expect("read kpop_multiturn_prompts.rs");
    assert!(
        src.contains("include!(\"kpop_multiturn_prompts_tests.inc\")"),
        "test setup: expected include! of kpop_multiturn_prompts_tests.inc"
    );
    assert_tracked_in_git("src/kpop_multiturn_prompts_tests.inc");
}
