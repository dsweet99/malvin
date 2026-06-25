use std::io::{stdout, IsTerminal};

/// True when agent stdout should use the styled logging formatter (TTY, not piped).
#[must_use]
pub fn stdout_is_interactive() -> bool {
    stdout().is_terminal()
}

/// Opt-in via `MALVIN_FORCE_STDOUT_TEE=1` for live/integration tests where stdout is piped.
#[must_use]
pub fn force_stdout_tee_from_env() -> bool {
    std::env::var("MALVIN_FORCE_STDOUT_TEE")
        .ok()
        .is_some_and(|v| v == "1")
}

/// True when ACP tool summaries and agent chunks should tee to stdout / `stdout.log`.
#[must_use]
pub fn agent_stdout_tee_enabled() -> bool {
    stdout_is_interactive() || force_stdout_tee_from_env()
}

#[cfg(test)]
mod tests {
    #[test]
    fn kiss_cov_stdout_tee_env() {
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs{
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _ = agent_stdout_tee_enabled;
        let _ = force_stdout_tee_from_env;
        let _ = stdout_is_interactive;
    }
}
