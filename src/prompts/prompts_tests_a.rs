use std::collections::HashMap;

use crate::prompts::*;

fn write_kpop_prompt_fixtures(root: &std::path::Path) {
    for (name, body) in [
        ("header.md", ""),
        ("kpop_common.md", "kc"),
        ("mpc_block_a.md", "ka"),
        ("mpc_block_b.md", "kb"),
        ("mpc_block_c.md", "kc"),
    ] {
        std::fs::write(root.join(name), body).unwrap();
    }
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
    write_kpop_prompt_fixtures(root);
    let store = PromptStore::with_root(root.to_path_buf());
    store
        .validate_kpop_prompts(crate::prompts::KpopPromptValidation {
            require_mbc2: false,
        })
        .expect("kpop-only ok");
    assert!(
        store.validate_required().is_err(),
        "full workflow should still require kpop_program when only header is present"
    );
}

#[test]
fn validate_kpop_prompts_does_not_require_mbc2_when_not_requested() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_kpop_prompt_fixtures(root);
    let store = PromptStore::with_root(root.to_path_buf());
    store
        .validate_kpop_prompts(crate::prompts::KpopPromptValidation {
            require_mbc2: false,
        })
        .expect("schedule without MBC2 should not require mbc2.md");
}

#[test]
fn validate_kpop_prompts_requires_mbc2_when_requested() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_kpop_prompt_fixtures(root);
    let store = PromptStore::with_root(root.to_path_buf());
    let err = store
        .validate_kpop_prompts(crate::prompts::KpopPromptValidation {
            require_mbc2: true,
        })
        .unwrap_err();
    assert!(
        err.0.contains("mbc2.md"),
        "expected mbc2 missing error, got {:?}",
        err.0
    );
    assert!(
        err.0.contains("priors.md"),
        "expected priors missing error, got {:?}",
        err.0
    );
}

#[test]
fn validate_kpop_prompts_requires_priors_when_mbc2_present() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_kpop_prompt_fixtures(root);
    std::fs::write(root.join("mbc2.md"), "mbc2").unwrap();
    let store = PromptStore::with_root(root.to_path_buf());
    let err = store
        .validate_kpop_prompts(crate::prompts::KpopPromptValidation {
            require_mbc2: true,
        })
        .unwrap_err();
    assert!(
        err.0.contains("priors.md"),
        "expected priors missing error, got {:?}",
        err.0
    );
    assert!(
        !err.0.contains("mbc2.md"),
        "mbc2 is present; error was {:?}",
        err.0
    );
}

#[test]
fn render_priors_mbc2_prompt_expands_paths_and_wraps_mbc2() {
    let store = PromptStore::default_store();
    let ctx = HashMap::from([
        (
            "user_request_path".to_string(),
            "./logs/run/user_request.md".to_string(),
        ),
        (
            "priors_path".to_string(),
            "./logs/run/_kpop/priors.md".to_string(),
        ),
    ]);
    let out = render_priors_mbc2_prompt(&store, &ctx).expect("render priors mbc2");
    assert!(out.contains("./logs/run/user_request.md"));
    assert!(out.contains("./logs/run/_kpop/priors.md"));
    assert!(out.contains("MBC2"));
    assert!(out.contains("Generate at least 30"));
    assert!(!out.contains("{{"));
}

#[test]
fn render_expands_coding_rules_placeholder_to_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::write(root.join("header.md"), "H").unwrap();
    std::fs::write(root.join("mpc_block_a.md"), "{{ coding_rules }}").unwrap();
    let store = PromptStore::with_root(root.to_path_buf());
    let out = store.render("mpc_block_a.md", &HashMap::new()).unwrap();
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
    std::fs::write(root.join("kpop_program.md"), "x").unwrap();
    let store = PromptStore::with_root(root.to_path_buf());
    let err = store.validate_required().unwrap_err();
    assert!(
        err.0.contains("header.md"),
        "expected missing header in error: {}",
        err.0
    );
}

#[test]
fn validate_required_fails_when_kpop_program_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    for &name in crate::prompts::REQUIRED_PROMPTS {
        if name == "kpop_program.md" {
            continue;
        }
        std::fs::write(root.join(name), "x").unwrap();
    }
    let store = PromptStore::with_root(root.to_path_buf());
    let err = store.validate_required().unwrap_err();
    assert!(
        err.0.contains("kpop_program.md"),
        "custom prompt roots must fail fast when kpop_program.md is absent: {}",
        err.0
    );
}

#[test]
fn default_kpop_program_avoids_mandated_tool_names() {
    let body = crate::prompts::default_file("kpop_program.md").expect("kpop_program.md");
    let banned = ["kiss", "pytest", "cargo clippy", "kiss-ai"];
    for needle in banned {
        assert!(
            !body.contains(needle),
            "kpop_program.md must not name mandated tools; found {needle:?} in template"
        );
    }
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
