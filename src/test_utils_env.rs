//! Process environment and cwd helpers for unit tests.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub use crate::test_poll::{
    test_post_teardown_poll_interval, test_post_teardown_wait_budget, test_wait_until_async,
};

fn stable_test_cwd() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Save cwd for restore; falls back to the crate root when the process cwd was deleted.
#[must_use]
pub fn save_cwd() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| {
        let stable = stable_test_cwd();
        std::env::set_current_dir(&stable).expect("chdir stable fallback");
        stable
    })
}

pub fn restore_cwd(path: &Path) {
    if std::env::set_current_dir(path).is_err() {
        let _ = std::env::set_current_dir(stable_test_cwd());
    }
}

/// Run an async test body on a lightweight current-thread Tokio runtime.
pub fn block_on_test_async<F, T>(future: F) -> T
where
    F: std::future::Future<Output = T>,
{
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("test runtime")
        .block_on(future)
}

/// Enable fast ACP teardown for tests that spawn sandbox children but do not exercise SIGTERM escalation.
pub fn enable_test_fast_teardown() {
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var(crate::acp::MALVIN_TEST_NO_REAL_AGENT_ENV, "1");
    }
}

/// Clears mock-agent env left behind by integration tests that skip restore.
pub fn clear_test_no_real_agent_env() {
    #[allow(unsafe_code)]
    unsafe {
        std::env::remove_var(crate::acp::MALVIN_TEST_NO_REAL_AGENT_ENV);
    }
}

/// Restores env vars saved at construction time.
pub struct SavedEnvVars {
    entries: Vec<(String, Option<OsString>)>,
}

impl SavedEnvVars {
    #[must_use]
    pub fn capture(names: &[&str]) -> Self {
        let entries = names
            .iter()
            .map(|name| ((*name).to_string(), std::env::var_os(name)))
            .collect();
        Self { entries }
    }
}

impl Drop for SavedEnvVars {
    fn drop(&mut self) {
        #[allow(unsafe_code)]
        unsafe {
            for (name, value) in self.entries.drain(..) {
                match value {
                    Some(v) => std::env::set_var(&name, v),
                    None => std::env::remove_var(&name),
                }
            }
        }
    }
}
