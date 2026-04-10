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
