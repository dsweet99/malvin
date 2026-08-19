#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]

use std::path::Path;
use std::sync::Mutex;

#[path = "test_utils_env.rs"]
mod env;
pub use env::{
    SavedEnvVars, allow_home_malvin_config_mutation_for_test, block_on_test_async,
    clear_test_no_real_agent_env, enable_test_fast_teardown, restore_cwd,
    revoke_home_malvin_config_mutation_for_test, save_cwd, set_test_home_env,
    test_post_teardown_poll_interval, test_post_teardown_wait_budget, test_wait_until_async,
};

pub static MALVIN_TEST_ENV_MUTEX: Mutex<()> = Mutex::new(());

pub fn test_env_lock() -> std::sync::MutexGuard<'static, ()> {
    MALVIN_TEST_ENV_MUTEX
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(test)]
pub fn with_cwd<T>(cwd: &std::path::Path, f: impl FnOnce() -> T) -> T {
    let old = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(cwd).expect("chdir");
    let out = f();
    std::env::set_current_dir(old).expect("restore");
    out
}

#[cfg(test)]
#[path = "test_isolated_home.rs"]
mod isolated_home;

#[cfg(test)]
pub use isolated_home::with_isolated_home;

#[cfg(test)]
pub fn empty_session_dotfile_backups(work: &Path) -> crate::artifacts::SessionDotfileBackups {
    crate::artifacts::SessionDotfileBackups::snapshot(work).expect("snapshot session dotfiles")
}
