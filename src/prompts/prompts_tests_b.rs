use std::collections::HashMap;

use crate::prompts::*;

fn write_nested_coding_rules_fixture(root: &std::path::Path) {
    std::fs::write(root.join("header.md"), "").unwrap();
    std::fs::write(
        root.join("bug_fix.md"),
        "START\n{{ coding_rules }}\nEND\n",
    )
    .unwrap();
    std::fs::write(root.join("coding_rules.md"), "Path={{ plan_path }}.\n").unwrap();
}

#[test]
fn coding_rules_nested_placeholders_expand() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_nested_coding_rules_fixture(root);
    let store = PromptStore::with_root(root.to_path_buf());
    let mut ctx = HashMap::new();
    ctx.insert("plan_path".to_string(), "/P".to_string());
    ctx.insert("quality_gates".to_string(), String::new());
    let out = store.render("bug_fix.md", &ctx).unwrap();
    assert!(
        out.contains("/P") && !out.contains("{{ plan_path }}"),
        "expected nested plan_path in coding_rules; got:\n{out}"
    );
}

fn store_with_header_rules_bug_fix(root: &std::path::Path) -> PromptStore {
    std::fs::write(root.join("header.md"), "OPENING").unwrap();
    std::fs::write(root.join("coding_rules.md"), "RULES").unwrap();
    std::fs::write(root.join("bug_fix.md"), "{{ coding_rules }}").unwrap();
    PromptStore::with_root(root.to_path_buf())
}

#[test]
fn header_prepends_coding_rules_placeholder() {
    let tmp = tempfile::tempdir().unwrap();
    let store = store_with_header_rules_bug_fix(tmp.path());
    let mut ctx = HashMap::new();
    ctx.insert("plan_path".to_string(), "/x".to_string());
    ctx.insert("kpop_log_dir".to_string(), "./_kpop".to_string());
    ctx.insert("quality_gates".to_string(), String::new());
    let out = store.render("bug_fix.md", &ctx).unwrap();
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
    std::fs::write(root.join("bug_fix.md"), "x {{ not_in_context }} y").unwrap();
    let store = PromptStore::with_root(root.to_path_buf());
    let err = store.render("bug_fix.md", &HashMap::new()).unwrap_err();
    assert!(
        err.0.contains("{{"),
        "expected brace rejection, got {:?}",
        err.0
    );
    assert!(
        err.0.contains("bug_fix.md"),
        "expected prompt file in error, got {:?}",
        err.0
    );
}

#[test]
fn enforce_no_unresolved_braces_in_reports_prompt_file() {
    let err = crate::prompts::enforce_no_unresolved_braces_in("x {{ y }} z", Some("kpop_program.md"))
        .expect_err("braces");
    assert!(err.0.contains("kpop_program.md"));
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

