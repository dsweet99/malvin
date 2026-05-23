use std::collections::HashMap;

use crate::prompts::*;

#[test]
fn validate_kpop_prompts_requires_learn_when_run_learn() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::write(root.join("header.md"), "").unwrap();
    std::fs::write(root.join("kpop_common.md"), "kc").unwrap();
    std::fs::write(root.join("kpop_block.md"), "kb").unwrap();
    let store = PromptStore::with_root(root.to_path_buf());
    let err = store
        .validate_kpop_prompts(crate::prompts::KpopPromptValidation {
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

fn write_nested_coding_rules_implement_fixture(root: &std::path::Path) {
    std::fs::write(root.join("header.md"), "").unwrap();
    std::fs::write(
        root.join("implement.md"),
        "START\n{{ coding_rules }}\nEND\n",
    )
    .unwrap();
    std::fs::write(root.join("coding_rules.md"), "Path={{ plan_path }}.\n").unwrap();
}

#[test]
fn coding_rules_nested_placeholders_expand() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_nested_coding_rules_implement_fixture(root);
    let store = PromptStore::with_root(root.to_path_buf());
    let mut ctx = HashMap::new();
    ctx.insert("plan_path".to_string(), "/P".to_string());
    ctx.insert("quality_gates".to_string(), String::new());
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
    ctx.insert("quality_gates".to_string(), String::new());
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
    assert!(crate::prompts::enforce_no_unresolved_braces("no templates").is_ok());
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

#[test]
fn learn_prompt_has_consistent_memory_target_guidance() {
    let learn = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/default_prompts/learn.md"
    ));
    assert!(
        learn.contains("Edit an `.malvin_memory/*.md` file"),
        "expected consistent memory-path guidance in learn prompt"
    );
    assert!(
        learn.contains("in one of `./.malvin_memory/*.md`"),
        "expected fallback memory file guidance in learn prompt"
    );
}

#[test]
fn learn_prompt_has_no_obvious_typo() {
    let learn = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/default_prompts/learn.md"
    ));
    assert!(
        !learn.contains("oncrement"),
        "expected learn typo to be fixed"
    );
}
