//! Isolated HOME + workspace cwd for unit tests (avoids parent-session ACP spawn locks).

use std::path::{Path, PathBuf};

struct IsolatedTestEnv {
    _tmp: tempfile::TempDir,
    old_home: Option<std::ffi::OsString>,
    old_cwd: PathBuf,
}

fn stable_fallback_cwd() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

impl IsolatedTestEnv {
    fn new() -> (Self, PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path().join("home");
        std::fs::create_dir_all(&home).unwrap();
        let old_home = std::env::var_os("HOME");
        {
            #[allow(unsafe_code)]
            unsafe {
                std::env::set_var("HOME", &home);
            }
        }
        let work = tmp.path().join("repo");
        std::fs::create_dir_all(&work).unwrap();
        let old_cwd = std::env::current_dir().unwrap_or_else(|_| {
            let stable = stable_fallback_cwd();
            std::env::set_current_dir(&stable).expect("chdir stable fallback");
            stable
        });
        std::env::set_current_dir(&work).expect("chdir isolated workspace");
        (
            Self {
                _tmp: tmp,
                old_home,
                old_cwd,
            },
            work,
        )
    }
}

impl Drop for IsolatedTestEnv {
    fn drop(&mut self) {
        if std::env::set_current_dir(&self.old_cwd).is_err() {
            let _ = std::env::set_current_dir(stable_fallback_cwd());
        }
        #[allow(unsafe_code)]
        unsafe {
            match self.old_home.take() {
                Some(h) => std::env::set_var("HOME", h),
                None => std::env::remove_var("HOME"),
            }
        }
    }
}

pub fn with_isolated_home<F>(f: F)
where
    F: FnOnce(&Path),
{
    let _lock = super::test_env_lock();
    let (_env, work) = IsolatedTestEnv::new();
    f(&work);
}
