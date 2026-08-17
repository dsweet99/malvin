use std::collections::HashMap;

use crate::prompts::*;

#[test]
fn substitute_replaces_dollar_keys() {
    let mut m = HashMap::new();
    m.insert("plan_path".to_string(), "/p".to_string());
    assert_eq!(
        crate::prompts::substitute_template("Hello $plan_path end", &m),
        "Hello /p end"
    );
}

#[test]
fn validate_required_ok_when_header_present() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::write(root.join("header.md"), "").unwrap();
    let store = PromptStore::with_root(root.to_path_buf());
    store.validate_required().expect("header is present");
}

#[test]
fn render_expands_coding_rules_placeholder_to_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::write(root.join("header.md"), "H").unwrap();
    std::fs::write(root.join("kpop_common.md"), "{{ coding_rules }}").unwrap();
    let store = PromptStore::with_root(root.to_path_buf());
    let out = store.render("kpop_common.md", &HashMap::new()).unwrap();
    assert_eq!(out, "");
}

#[test]
fn load_header_swallows_missing_prompt_file() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let store = PromptStore::with_root(root.to_path_buf());
    assert_eq!(store.load_header(), "");
}

#[test]
fn validate_required_fails_when_header_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let store = PromptStore::with_root(root.to_path_buf());
    let err = store.validate_required().unwrap_err();
    assert!(
        err.0.contains("header.md"),
        "expected missing header in error: {}",
        err.0
    );
}

#[test]
fn validate_required_rejects_directory_in_place_of_prompt_file() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    for &name in crate::prompts::REQUIRED_PROMPTS {
        std::fs::create_dir_all(root.join(name)).unwrap();
    }
    let store = PromptStore::with_root(root.to_path_buf());
    let err = store.validate_required().unwrap_err();
    for name in crate::prompts::REQUIRED_PROMPTS {
        assert!(
            err.0.contains(name),
            "missing required prompt {name} in {err:?}"
        );
    }
}
