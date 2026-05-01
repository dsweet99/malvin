use std::collections::HashMap;
mod user_home_dir_tests;

use super::*;

#[test]
fn substitute_replaces_dollar_keys() {
    let mut m = HashMap::new();
    m.insert("plan_path".to_string(), "/p".to_string());
    assert_eq!(
        super::substitute_template("Hello $plan_path end", &m),
        "Hello /p end"
    );
}

#[test]
fn validate_kpop_prompts_ok_with_only_kpop_while_full_set_would_fail() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::write(root.join("header.md"), "").unwrap();
    std::fs::write(root.join("kpop_common.md"), "kc").unwrap();
    std::fs::write(root.join("kpop_block.md"), "kb").unwrap();
    let store = PromptStore::with_root(root.to_path_buf());
    store
        .validate_kpop_prompts(super::KpopPromptValidation {
            run_learn: false,
            require_mbc2: false,
        })
        .expect("kpop-only ok");
    assert!(
        store.validate_required().is_err(),
        "full workflow should still require implement/review/etc."
    );
}

#[test]
fn validate_kpop_prompts_does_not_require_mbc2_when_not_requested() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::write(root.join("header.md"), "").unwrap();
    std::fs::write(root.join("kpop_common.md"), "kc").unwrap();
    std::fs::write(root.join("kpop_block.md"), "kb").unwrap();
    let store = PromptStore::with_root(root.to_path_buf());
    store
        .validate_kpop_prompts(super::KpopPromptValidation {
            run_learn: false,
            require_mbc2: false,
        })
        .expect("schedule without MBC2 should not require mbc2.md");
}

#[test]
fn validate_kpop_prompts_requires_mbc2_when_requested() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::write(root.join("header.md"), "").unwrap();
    std::fs::write(root.join("kpop_common.md"), "kc").unwrap();
    std::fs::write(root.join("kpop_block.md"), "kb").unwrap();
    let store = PromptStore::with_root(root.to_path_buf());
    let err = store
        .validate_kpop_prompts(super::KpopPromptValidation {
            run_learn: false,
            require_mbc2: true,
        })
        .unwrap_err();
    assert!(
        err.0.contains("mbc2_pure.md"),
        "expected mbc2_pure missing error, got {:?}",
        err.0
    );
}

#[test]
fn kpop_validation_may_omit_coding_rules_without_error() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::write(root.join("header.md"), "H").unwrap();
    std::fs::write(root.join("kpop_common.md"), "kc").unwrap();
    std::fs::write(root.join("kpop_block.md"), "{{ coding_rules }}").unwrap();
    let store = PromptStore::with_root(root.to_path_buf());
    let validation = store.validate_kpop_prompts(super::KpopPromptValidation {
        run_learn: false,
        require_mbc2: false,
    });
    assert!(validation.is_ok(), "kpop validation should unexpectedly pass: {validation:?}");
    let out = store.render("kpop_block.md", &HashMap::new()).unwrap();
    assert_eq!(out, "H");
}

#[test]
fn load_coding_rules_swallows_missing_prompt_file() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::write(root.join("header.md"), "H").unwrap();
    let store = PromptStore::with_root(root.to_path_buf());
    assert_eq!(store.load_coding_rules(), "");
}

#[test]
fn load_header_swallows_missing_prompt_file() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let store = PromptStore::with_root(root.to_path_buf());
    assert_eq!(store.load_header(), "");
}

#[test]
fn validate_required_fails_when_header_or_coding_rules_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    for name in ["implement.md", "review_1.md", "review_2.md", "concerns.md"] {
        std::fs::write(root.join(name), "x").unwrap();
    }
    let store = PromptStore::with_root(root.to_path_buf());
    let err = store.validate_required().unwrap_err();
    assert!(
        err.0.contains("header.md") && err.0.contains("coding_rules.md"),
        "expected missing header + coding_rules in error: {}",
        err.0
    );
}

#[test]
fn validate_required_rejects_directory_in_place_of_prompt_file() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    for &name in super::REQUIRED_PROMPTS {
        std::fs::create_dir_all(root.join(name)).unwrap();
    }
    let store = PromptStore::with_root(root.to_path_buf());
    let err = store.validate_required().unwrap_err();
    for name in super::REQUIRED_PROMPTS {
        assert!(err.0.contains(name), "missing required prompt {name} in {err:?}");
    }
}

#[test]
fn validate_kpop_prompts_requires_learn_when_run_learn() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::write(root.join("header.md"), "").unwrap();
    std::fs::write(root.join("kpop_common.md"), "kc").unwrap();
    std::fs::write(root.join("kpop_block.md"), "kb").unwrap();
    let store = PromptStore::with_root(root.to_path_buf());
    let err = store
        .validate_kpop_prompts(super::KpopPromptValidation {
            run_learn: true,
            require_mbc2: false,
        })
        .unwrap_err();
    assert!(
        err.0.contains("learn.md"),
        "expected learn missing error, got {:?}",
        err.0
    );
}

#[test]
fn coding_rules_nested_placeholders_expand() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::write(root.join("header.md"), "").unwrap();
    std::fs::write(
        root.join("implement.md"),
        "START\n{{ coding_rules }}\nEND\n",
    )
    .unwrap();
    std::fs::write(root.join("coding_rules.md"), "Path={{ plan_path }}.\n").unwrap();
    let store = PromptStore::with_root(root.to_path_buf());
    let mut ctx = HashMap::new();
    ctx.insert("plan_path".to_string(), "/P".to_string());
    let out = store.render("implement.md", &ctx).unwrap();
    assert!(
        out.contains("/P") && !out.contains("{{ plan_path }}"),
        "expected nested plan_path in coding_rules; got:\n{out}"
    );
}

fn store_with_header_rules_implement(root: &std::path::Path) -> PromptStore {
    std::fs::write(root.join("header.md"), "OPENING").unwrap();
    std::fs::write(root.join("coding_rules.md"), "RULES").unwrap();
    std::fs::write(root.join("implement.md"), "{{ coding_rules }}").unwrap();
    PromptStore::with_root(root.to_path_buf())
}

#[test]
fn header_prepends_coding_rules_placeholder() {
    let tmp = tempfile::tempdir().unwrap();
    let store = store_with_header_rules_implement(tmp.path());
    let mut ctx = HashMap::new();
    ctx.insert("plan_path".to_string(), "/x".to_string());
    ctx.insert("kpop_log_dir".to_string(), "./_kpop".to_string());
    let out = store.render("implement.md", &ctx).unwrap();
    assert!(
        out.starts_with("OPENING\n\nRULES"),
        "expected header before rules; got:\n{out}"
    );
}

#[test]
fn render_prompt_only_skips_coding_rules_injection() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::write(root.join("header.md"), "H{{ plan_path }}").unwrap();
    std::fs::write(root.join("coding_rules.md"), "SHOULD_NOT_APPEAR").unwrap();
    let store = PromptStore::with_root(root.to_path_buf());
    let mut ctx = HashMap::new();
    ctx.insert("plan_path".to_string(), "/p".to_string());
    let out = store.render_prompt_only("header.md", &ctx).unwrap();
    assert_eq!(out, "H/p");
}

#[test]
fn render_fails_when_double_brace_remains() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::write(root.join("header.md"), "").unwrap();
    std::fs::write(root.join("coding_rules.md"), "").unwrap();
    std::fs::write(root.join("implement.md"), "x {{ not_in_context }} y").unwrap();
    let store = PromptStore::with_root(root.to_path_buf());
    let err = store.render("implement.md", &HashMap::new()).unwrap_err();
    assert!(
        err.0.contains("{{"),
        "expected brace rejection, got {:?}",
        err.0
    );
}

#[test]
fn enforce_no_unresolved_braces_ok_when_clean() {
    assert!(super::enforce_no_unresolved_braces("no templates").is_ok());
}

#[test]
fn merge_header_and_coding_rules_combines_nonempty() {
    assert_eq!(
        merge_header_and_coding_rules("head", "rules"),
        "head\n\nrules"
    );
    assert_eq!(
        merge_header_and_coding_rules("  head  ", "  rules  "),
        "head\n\nrules"
    );
}

#[test]
fn merge_header_and_coding_rules_handles_empty() {
    assert_eq!(merge_header_and_coding_rules("", ""), "");
    assert_eq!(merge_header_and_coding_rules("head", ""), "head");
    assert_eq!(merge_header_and_coding_rules("", "rules"), "rules");
    assert_eq!(merge_header_and_coding_rules("  ", "  "), "");
}

