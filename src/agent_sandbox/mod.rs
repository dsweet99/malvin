mod bridge;
pub(crate) mod feasibility;
pub mod spawn;
mod teardown;

pub use teardown::AgentSandboxGuard;

use std::path::Path;

pub(crate) fn sandbox_test_no_real_agent_enabled() -> bool {
    std::env::var_os("MALVIN_TEST_NO_REAL_AGENT")
        .is_some_and(|v| !v.is_empty() && v != "0")
}
use crate::output::print_log_warning;

pub fn use_microsandbox_for_spawn(no_sandbox: bool, _work_dir: &Path) -> bool {
    if no_sandbox || sandbox_test_no_real_agent_enabled() {
        return false;
    }
    let Some(agent) = crate::support_paths::agent_or_cursor_agent_bin() else {
        return false;
    };
    if feasibility::linux_node_in_bundle(&agent).is_some() {
        return true;
    }
    print_log_warning(
        "microsandbox: no Linux ELF agent node; running agent on host (use --no-sandbox to silence)",
    );
    false
}

pub fn load_mem_config(work_dir: &Path) -> crate::agent_sandbox_config::AgentSandboxConfig {
    crate::agent_sandbox_config::load_agent_sandbox_config(work_dir)
}

#[cfg(test)]
#[path = "kiss_tests.rs"]
mod kiss_tests;

