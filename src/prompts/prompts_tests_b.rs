use std::collections::HashMap;

use crate::prompts::*;

#[test]
fn render_prompt_only_does_not_inject_coding_rules() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::write(root.join("header.md"), "H{{ plan_path }}").unwrap();
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
    let err = crate::prompts::enforce_no_unresolved_braces_in("x {{ y }} z", Some("header.md"))
        .expect_err("braces");
    assert!(err.0.contains("header.md"));
}

#[test]
fn enforce_no_unresolved_braces_ok_when_clean() {
    assert!(crate::prompts::enforce_no_unresolved_braces("no templates").is_ok());
}

#[test]
fn enforce_no_unresolved_braces_allows_user_content_with_non_placeholder_braces() {
    assert!(crate::prompts::enforce_no_unresolved_braces(
        "docs mention unresolved `{{…}}` or literal `{{ before ACP`"
    )
    .is_ok());
}

#[test]
fn render_prompt_only_allows_user_request_values_with_double_braces() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::write(
        root.join("kpop_block.md"),
        "User request:\n\n{{ user_request }}",
    )
    .unwrap();
    let store = PromptStore::with_root(root.to_path_buf());
    let mut ctx = HashMap::new();
    ctx.insert(
        "user_request".to_string(),
        "Expand {{ code_extra }} in router_b_*".to_string(),
    );
    let out = store.render_prompt_only("kpop_block.md", &ctx).unwrap();
    assert!(out.contains("Expand {{ code_extra }}"));
}

#[test]
fn enforce_template_placeholders_resolved_rejects_missing_keys_in_template() {
    let err = crate::prompts::enforce_template_placeholders_resolved_in(
        "x {{ not_in_context }} y",
        &HashMap::new(),
        Some("bug_fix.md"),
    )
    .expect_err("missing template key");
    assert!(err.0.contains("bug_fix.md"));
}

#[test]
fn render_header_expands_header_placeholders() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::write(root.join("header.md"), "Path={{ plan_path }}.\n").unwrap();
    let store = PromptStore::with_root(root.to_path_buf());
    let mut ctx = HashMap::new();
    ctx.insert("plan_path".to_string(), "/P".to_string());
    let out = render_header(&store, &ctx).unwrap();
    assert!(
        out.contains("/P") && !out.contains("{{ plan_path }}"),
        "expected plan_path in header; got:\n{out}"
    );
}
