#![cfg_attr(test, allow(unsafe_code))]

pub(crate) mod sandbox_oom;
pub(crate) mod acp_spawn_lock;
pub mod acp_spawn_sweep;
pub mod malvin_sandbox;
#[cfg(test)]
#[path = "malvin_sandbox_tests.rs"]
pub(crate) mod malvin_sandbox_tests;
pub(crate) mod parent_death_signal;
pub mod process_group_rss;
pub(crate) mod active_agent_heartbeat;
pub(crate) mod child_health;

mod jsonl_trace;
mod outgoing_prompt_trace;
pub use outgoing_prompt_trace::CoderPromptOptions;

pub(crate) use jsonl_trace::AcpJsonlTrace;

#[cfg(unix)]
#[path = "unix_process_ancestor.rs"]
mod unix_process_ancestor;
#[cfg(unix)]
#[path = "unix_process_group_kill_targets.rs"]
mod unix_process_group_kill_targets;
#[path = "unix_process_group_ps.rs"]
mod unix_process_group_ps;
#[path = "unix_process_group_teardown.rs"]
mod unix_process_group_teardown;
#[cfg(unix)]
#[path = "unix_process_group_teardown_poll.rs"]
mod unix_process_group_teardown_poll;
#[cfg(unix)]
#[path = "unix_sandbox_monitor.rs"]
mod unix_sandbox_monitor;
#[cfg(unix)]
pub(crate) use unix_process_ancestor::is_ancestor_pid;
#[cfg(unix)]
pub(crate) use unix_process_group_kill_targets::{
    clear_session_spawn_affiliation, refresh_session_spawn_affiliation,
};
#[cfg(unix)]
pub(crate) use unix_process_group_ps::pid_alive;
pub use unix_process_group_ps::{signal_process_group, snapshot_pids, spawned_pids_since_baseline};
pub use unix_process_group_teardown::{
    reap_baseline_amnestied_agent_orphans_blocking, terminate_agent_process_group,
    terminate_process_group,
};
#[cfg(unix)]
pub use unix_sandbox_monitor::sandbox_monitor_pids;

mod process_group_mem_watch;
#[cfg(unix)]
pub use process_group_mem_watch::{MemWatchHandles, watch_process_group_memory};

#[path = "process_group_terminate.rs"]
mod process_group_terminate;
#[cfg(unix)]
pub(crate) use process_group_terminate::{
    terminate_agent_process_group_blocking, terminate_agent_process_group_for_interrupt,
};

#[path = "coalesce.rs"]
mod coalesce;
pub(crate) use coalesce::*;

#[path = "coalesce_trace.rs"]
mod coalesce_trace;
pub(crate) use coalesce_trace::*;

#[path = "wrap_agent_bundle.rs"]
mod wrap_agent_bundle;
#[path = "wrap_retry_policy.rs"]
mod wrap_retry_policy;
pub(crate) use wrap_agent_bundle::*;
pub use wrap_agent_bundle::{AgentError, AgentFault, AgentIoOptions, AuthError};
pub(crate) use wrap_retry_policy::*;

#[path = "agent_helpers.rs"]
mod agent_helpers;
pub use agent_helpers::*;

#[path = "backoff.rs"]
mod backoff;
pub(crate) use backoff::backoff_after_agent_failure;

#[cfg(all(unix, any(test, debug_assertions)))]
#[path = "hostile_orphan_test_util.rs"]
pub mod hostile_orphan_test_util;

#[path = "acp_spawn_lock_peer.rs"]
pub(super) mod acp_spawn_lock_peer;
#[path = "acp_spawn_lock_probe.rs"]
pub(super) mod acp_spawn_lock_probe;
#[cfg(target_os = "linux")]
#[path = "process_group_rss/linux.rs"]
pub(super) mod process_group_rss_linux;
#[cfg(target_os = "macos")]
#[path = "process_group_rss/macos.rs"]
pub(super) mod process_group_rss_macos;
#[cfg(target_os = "linux")]
#[path = "child_health/linux.rs"]
pub(super) mod child_health_linux;
#[cfg(target_os = "macos")]
#[path = "child_health/macos.rs"]
pub(super) mod child_health_macos;
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
#[path = "child_health/other.rs"]
pub(crate) mod child_health_other;
#[cfg(unix)]
#[cfg(all(test, unix))]
#[path = "unix_process_ancestor_tests.rs"]
pub(super) mod unix_process_ancestor_tests;
#[cfg(unix)]
#[path = "session_spawn_affiliation.rs"]
pub(super) mod session_spawn_affiliation;
#[cfg(unix)]
#[path = "unix_process_group_ps_proc.rs"]
pub(super) mod unix_process_group_ps_proc;
#[cfg(all(test, unix))]
#[path = "unix_process_group_ps_tests.rs"]
pub(crate) mod unix_process_group_ps_tests;
#[cfg(all(test, unix))]
#[path = "unix_process_group_teardown_escalation_tests.rs"]
pub(super) mod unix_process_group_teardown_escalation_tests;
#[cfg(all(test, unix))]
#[path = "unix_process_group_teardown_tests.rs"]
pub(crate) mod unix_process_group_teardown_tests;
#[cfg(unix)]
#[path = "unix_process_group_teardown_timing.rs"]
pub(super) mod unix_process_group_teardown_timing;
#[cfg(all(test, unix))]
#[path = "process_group_mem_watch_policy_tests.rs"]
pub(super) mod process_group_mem_watch_policy_tests;
#[path = "attempt_ceiling.rs"]
pub(super) mod attempt_ceiling;
#[path = "retry_teardown.rs"]
pub(super) mod retry_teardown;
#[cfg(all(unix, any(test, debug_assertions)))]
#[path = "hostile_orphan_read_pid.rs"]
pub(super) mod hostile_orphan_read_pid;
#[cfg(all(unix, any(test, debug_assertions)))]
#[path = "hostile_orphan_user_shell.rs"]
pub(super) mod hostile_orphan_user_shell;
