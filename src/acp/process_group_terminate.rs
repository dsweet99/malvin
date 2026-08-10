//! Process-group teardown helpers used by SDK sessions and sandbox interrupt.

use std::collections::HashSet;

#[cfg(unix)]
pub(crate) fn terminate_agent_process_group_blocking(
    process_group_id: Option<u32>,
    spawn_baseline: &HashSet<u32>,
) {
    super::unix_process_group_teardown::teardown_agent_sandbox_blocking(
        process_group_id,
        spawn_baseline,
    );
}

/// CTRL-C: SIGKILL without waiting.
#[cfg(unix)]
pub(crate) fn terminate_agent_process_group_for_interrupt(
    process_group_id: Option<u32>,
    spawn_baseline: &HashSet<u32>,
) {
    super::unix_process_group_teardown::teardown_agent_sandbox_for_interrupt(
        process_group_id,
        spawn_baseline,
    );
}
