#![cfg_attr(test, allow(unsafe_code))]

mod import_prelude;
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
    clear_session_spawn_affiliation, note_session_affiliated_pid, refresh_session_spawn_affiliation,
};
#[cfg(all(unix, test))]
pub(crate) use unix_process_group_kill_targets::{
    clear_session_spawn_affiliation_for_test, is_session_affiliated_pid,
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
pub use process_group_mem_watch::{
    MemWatchHandles, watch_process_group_memory,
};

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
pub(crate) use agent_helpers::*;

#[path = "backoff.rs"]
mod backoff;
pub(crate) use backoff::backoff_after_agent_failure;

#[cfg(unix)]
#[path = "hostile_orphan_test_util.rs"]
pub mod hostile_orphan_test_util;
