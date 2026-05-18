use std::collections::HashMap;

use crate::prompts::*;

#[test]
fn kiss_stringify_prompts_guards() {
    let _ = stringify!(crate::prompts::enforce_no_unresolved_braces);
    let _ = stringify!(crate::prompts::PromptError);
    let _ = stringify!(crate::prompts::KpopPromptValidation);
    let _ = stringify!(crate::prompts::merged_coding_rules);
    let _ = stringify!(crate::prompts::render_mbc2_for_scheduled_kpop_block);
}

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
fn validate_kpop_prompts_ok_with_only_kpop_while_full_set_would_fail() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::write(root.join("header.md"), "").unwrap();
    std::fs::write(root.join("kpop_common.md"), "kc").unwrap();
    std::fs::write(root.join("kpop_block.md"), "kb").unwrap();
    let store = PromptStore::with_root(root.to_path_buf());
    store
        .validate_kpop_prompts(crate::prompts::KpopPromptValidation {
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
        .validate_kpop_prompts(crate::prompts::KpopPromptValidation {
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
        .validate_kpop_prompts(crate::prompts::KpopPromptValidation {
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
    let validation = store.validate_kpop_prompts(crate::prompts::KpopPromptValidation {
        run_learn: false,
        require_mbc2: false,
    });
    assert!(
        validation.is_ok(),
        "kpop validation should unexpectedly pass: {validation:?}"
    );
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
    for name in [
        "implement.md",
        "reviewers_spawn.md",
        "review_write.md",
        "concerns.md",
    ] {
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
fn validate_required_fails_when_reviewers_spawn_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    for &name in crate::prompts::REQUIRED_PROMPTS {
        if name == "reviewers_spawn.md" {
            continue;
        }
        std::fs::write(root.join(name), "x").unwrap();
    }
    let store = PromptStore::with_root(root.to_path_buf());
    let err = store.validate_required().unwrap_err();
    assert!(
        err.0.contains("reviewers_spawn.md"),
        "custom prompt roots must fail fast when reviewers_spawn.md is absent: {}",
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
