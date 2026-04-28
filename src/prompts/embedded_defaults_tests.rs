#![allow(unsafe_code)]

use std::collections::HashMap;

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
fn default_store_uses_embedded_prompts_when_home_unset() {
    let prompt = default_store_with_unset_home();
    assert!(prompt.contains("Implement"));
}

fn default_store_with_unset_home() -> String {
    let (store, context) = default_prompt_store_with_unset_home();
    render_default_implement(&store, &context)
}

fn default_prompt_store_with_unset_home() -> (super::PromptStore, HashMap<String, String>) {
    let _lock = crate::test_utils::test_env_lock();
    let profile = tempfile::tempdir().unwrap().path().join("profile");
    std::fs::create_dir_all(&profile).unwrap();
    let _guard = with_unset_home_profile(profile);
    let store = {
        let store = super::PromptStore::default_store();
        store.ensure_defaults().unwrap();
        store
    };
    let context = HashMap::from([
        ("plan_path".to_string(), "/p".to_string()),
        ("grounding_path".to_string(), "/g".to_string()),
        ("result_path".to_string(), "/r".to_string()),
    ]);
    (store, context)
}

fn with_unset_home_profile(profile: std::path::PathBuf) -> EnvHomeGuard {
    let guard = EnvHomeGuard {
        home: std::env::var_os("HOME"),
        userprofile: std::env::var_os("USERPROFILE"),
    };
    unsafe {
        std::env::remove_var("HOME");
        std::env::set_var("USERPROFILE", profile);
    }
    guard
}

fn render_default_implement(
    store: &super::PromptStore,
    context: &HashMap<String, String>,
) -> String {
    store.render("implement.md", context).expect("render")
}

#[test]
fn kiss_stringify_embedded_defaults_units() {
    let _ = stringify!(crate::prompts::embedded_defaults_tests::EnvHomeGuard);
    let _ = stringify!(crate::prompts::embedded_defaults_tests::default_prompt_store_with_unset_home);
    let _ = stringify!(crate::prompts::embedded_defaults_tests::with_unset_home_profile);
    let _ = stringify!(crate::prompts::embedded_defaults_tests::render_default_implement);
}
