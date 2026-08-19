use std::io::{IsTerminal, stdout};

#[must_use]
pub fn stdout_is_interactive() -> bool {
    stdout().is_terminal()
}

#[must_use]
pub fn force_stdout_tee_from_env() -> bool {
    std::env::var("MALVIN_FORCE_STDOUT_TEE")
        .ok()
        .is_some_and(|v| v == "1")
}

#[must_use]
pub fn agent_stdout_tee_enabled() -> bool {
    stdout_is_interactive() || force_stdout_tee_from_env()
}

#[cfg(test)]
mod tests {
    use super::{agent_stdout_tee_enabled, force_stdout_tee_from_env, stdout_is_interactive};
    use crate::test_utils::test_env_lock;

    #[test]
    fn force_stdout_tee_from_env_requires_exact_one() {
        let _lock = test_env_lock();
        let prior = std::env::var_os("MALVIN_FORCE_STDOUT_TEE");
        #[allow(unsafe_code)]
        unsafe {
            std::env::remove_var("MALVIN_FORCE_STDOUT_TEE");
        }
        assert!(!force_stdout_tee_from_env());
        #[allow(unsafe_code)]
        unsafe {
            std::env::set_var("MALVIN_FORCE_STDOUT_TEE", "0");
        }
        assert!(!force_stdout_tee_from_env());
        #[allow(unsafe_code)]
        unsafe {
            std::env::set_var("MALVIN_FORCE_STDOUT_TEE", "1");
        }
        assert!(force_stdout_tee_from_env());
        assert!(agent_stdout_tee_enabled());
        let _ = stdout_is_interactive();
        #[allow(unsafe_code)]
        unsafe {
            match prior {
                Some(v) => std::env::set_var("MALVIN_FORCE_STDOUT_TEE", v),
                None => std::env::remove_var("MALVIN_FORCE_STDOUT_TEE"),
            }
        }
    }
}
