use std::collections::HashMap;

use super::*;

struct EnvHomeGuard {
    home: Option<std::ffi::OsString>,
    userprofile: Option<std::ffi::OsString>,
}

impl Drop for EnvHomeGuard {
    fn drop(&mut self) {
        unsafe {
            match self.home.take() {
                Some(v) => std::env::set_var("HOME", v),
                None => std::env::remove_var("HOME"),
            }
            match self.userprofile.take() {
                Some(v) => std::env::set_var("USERPROFILE", v),
                None => std::env::remove_var("USERPROFILE"),
            }
        }
    }
}

#[test]
fn default_store_uses_userprofile_when_home_unset() {
    let _lock = crate::test_utils::test_env_lock();
    let tmp = tempfile::tempdir().unwrap();
    let profile = tmp.path().join("profile");
    std::fs::create_dir_all(&profile).unwrap();
    let _guard = EnvHomeGuard {
        home: std::env::var_os("HOME"),
        userprofile: std::env::var_os("USERPROFILE"),
    };
    unsafe {
        std::env::remove_var("HOME");
        std::env::set_var("USERPROFILE", &profile);
    }
    let store = PromptStore::default_store();
    store.ensure_defaults().unwrap();
    assert!(
        profile
            .join(".malvin")
            .join("prompts")
            .join("implement.md")
            .is_file()
    );
}

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
    std::fs::write(root.join("kpop.md"), "kpop").unwrap();
    let store = PromptStore::with_root(root.to_path_buf());
    store
        .validate_kpop_prompts(false, 0.0)
        .expect("kpop-only ok");
    assert!(
        store.validate_required().is_err(),
        "full workflow should still require implement/review/etc."
    );
}

#[test]
fn validate_kpop_prompts_does_not_require_mbc2_for_positive_infinity() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::write(root.join("kpop.md"), "kpop").unwrap();
    let store = PromptStore::with_root(root.to_path_buf());
    store
        .validate_kpop_prompts(false, f64::INFINITY)
        .expect("non-finite p_creative should not imply MBC2");
}

#[test]
fn validate_kpop_prompts_requires_mbc2_when_p_creative_positive() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::write(root.join("kpop.md"), "kpop").unwrap();
    let store = PromptStore::with_root(root.to_path_buf());
    let err = store.validate_kpop_prompts(false, 0.1).unwrap_err();
    assert!(
        err.0.contains("mbc2.md"),
        "expected mbc2 missing error, got {:?}",
        err.0
    );
}

#[test]
fn validate_kpop_prompts_requires_learn_when_run_learn() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::write(root.join("kpop.md"), "kpop").unwrap();
    let store = PromptStore::with_root(root.to_path_buf());
    let err = store.validate_kpop_prompts(true, 0.0).unwrap_err();
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
