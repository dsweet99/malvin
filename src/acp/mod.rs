#![cfg_attr(test, allow(unsafe_code))]
//! Agent Client Protocol (`agent acp`) JSON-RPC over stdio.

mod import_prelude;
mod jsonl_trace;
mod outgoing_prompt_trace;
pub use outgoing_prompt_trace::CoderPromptOptions;
mod session_types;

#[path = "prompt_trace_writer.rs"]
mod prompt_trace_writer;
pub(crate) use prompt_trace_writer::LivePromptTraceArgs;
#[cfg(test)]
pub(crate) use prompt_trace_writer::open_kpop_timestamp_trace_writer;

#[path = "wrap_handshake_types.rs"]
mod wrap_handshake_types;
#[path = "wrap_session_io.rs"]
mod wrap_session_io;
#[path = "wrap_session_channels.rs"]
mod wrap_session_channels;
pub(crate) use wrap_handshake_types::*;
pub(crate) use wrap_session_channels::*;
pub(crate) use wrap_session_io::*;

#[cfg(test)]
#[path = "session_types_tests.rs"]
mod session_types_tests;

#[path = "unix_process_group_ps.rs"] mod unix_process_group_ps;
#[cfg(unix)] #[path = "unix_process_ancestor.rs"] mod unix_process_ancestor;
#[cfg(unix)] #[path = "unix_process_group_kill_targets.rs"] mod unix_process_group_kill_targets;
#[path = "unix_process_group_teardown.rs"] mod unix_process_group_teardown;
#[cfg(unix)]
#[path = "unix_process_group_teardown_poll.rs"]
mod unix_process_group_teardown_poll;
#[cfg(unix)]
#[path = "unix_sandbox_monitor.rs"] mod unix_sandbox_monitor;
pub use unix_process_group_ps::{snapshot_pids, spawned_pids_since_baseline, signal_process_group};
pub use unix_process_group_teardown::{
    reap_baseline_amnestied_agent_orphans_blocking, terminate_agent_process_group,
    terminate_process_group,
};
#[cfg(unix)] pub(crate) use unix_process_ancestor::is_ancestor_pid;
#[cfg(unix)] pub(crate) use unix_process_group_ps::pid_alive;
#[cfg(unix)] pub use unix_sandbox_monitor::sandbox_monitor_pids;
#[cfg(unix)]
pub(crate) use unix_process_group_kill_targets::{
    clear_session_spawn_affiliation, refresh_session_spawn_affiliation,
};
mod process_group_mem_watch;
#[cfg(unix)] pub use process_group_mem_watch::{MemWatchHandles, watch_process_group_memory};

pub(crate) use jsonl_trace::AcpJsonlTrace;
#[cfg(test)]
pub(crate) use session_types::AcpSessionInner;
pub use session_types::{AcpSession, AcpSpawnArgs};
pub(crate) use session_types::{PromptTraceWriter, ResponseTx};

#[path = "cursor_credentials.rs"]
mod cursor_credentials;
pub(crate) use cursor_credentials::*;

#[path = "coalesce.rs"]
mod coalesce;
pub(crate) use coalesce::*;

#[path = "coalesce_trace.rs"]
mod coalesce_trace;
pub(crate) use coalesce_trace::*;

#[path = "trace_line_write_tee.rs"]
mod trace_line_write_tee;
mod trace_plain_tee;
#[cfg(test)]
pub(crate) use trace_line_write_tee::format_styled_tool_summary_tee_line;
#[path = "trace_line_write_tool_summary.rs"]
mod trace_line_write_tool_summary;
pub(crate) mod trace_line_write;
pub(crate) use trace_line_write::{
    ReaderTraceLineOpts, reader_loop_verbose_and_trace_line, trace_file_write_line,
};
#[cfg(test)]
pub(crate) use trace_line_write_tool_summary::write_tool_summary_trace_line;

#[path = "session_trace_setup.rs"]
mod session_trace_setup;
#[path = "session_prompts_log.rs"]
mod session_prompts_log;
#[cfg(test)]
#[path = "session_trace_tests.rs"]
mod session_trace_tests;
pub(crate) use session_prompts_log::*;
pub(crate) use session_trace_setup::*;

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::Notify;

pub(crate) fn note_acp_trace_activity(
    acp_activity_seq: &Arc<AtomicU64>,
    acp_activity_notify: &Arc<Notify>,
) {
    acp_activity_seq.fetch_add(1, Ordering::SeqCst);
    acp_activity_notify.notify_waiters();
}

#[path = "transport/jsonrpc_error.rs"]
mod jsonrpc_error;
#[path = "transport/command.rs"]
mod command;
#[path = "transport/rpc_part1.rs"]
mod rpc_part1;
#[path = "transport/rpc_wait_args.rs"]
mod rpc_wait_args;
pub(crate) use rpc_wait_args::RpcWaitArgs;
#[path = "transport/rpc_part2.rs"]
mod rpc_part2;
#[path = "transport/rpc_part2_health.rs"]
mod rpc_part2_health;
#[path = "transport/handshake.rs"]
mod handshake;
pub(crate) use command::*;
pub(crate) use handshake::*;
pub(crate) use jsonrpc_error::*;
pub(crate) use rpc_part1::*;
pub(crate) use rpc_part2::*;

#[cfg(test)]
pub(crate) fn acp_rpc_timeout() -> std::time::Duration {
    std::time::Duration::from_secs(crate::config::acp_rpc_timeout_secs_from_env())
}

/// Whether the ACP handshake must send `authenticate` (`methodId: cursor_login`).
///
/// One-time `agent login` is enough for the CLI; when credentials are already present we skip the
/// redundant RPC (often ~10s+) and go straight to `session/new`.
pub(crate) fn requires_cursor_login_auth(
    explicit_api_key: Option<&str>,
    explicit_auth_token: Option<&str>,
) -> bool {
    if effective_cursor_api_key(explicit_api_key).is_some()
        || effective_cursor_auth_token(explicit_auth_token).is_some()
    {
        return false;
    }
    !crate::acp::cursor_cli_auth_established()
}

#[test]
fn acp_rpc_timeout_and_login_auth_smoke() {
    assert!(acp_rpc_timeout().as_secs() > 0);
    assert!(!requires_cursor_login_auth(Some("key"), Some("token")));
}

#[path = "wrap_reader_a.rs"]
mod wrap_reader_a;
pub(crate) use wrap_reader_a::*;

#[path = "wrap_reader_b.rs"]
mod wrap_reader_b;
pub(crate) use wrap_reader_b::*;

#[path = "wrap_session_spawn.rs"]
mod wrap_session_spawn;
pub(crate) use wrap_session_spawn::*;

#[path = "wrap_session_prompt.rs"]
mod wrap_session_prompt;
pub(crate) use wrap_session_prompt::*;

#[path = "wrap_session_post.rs"]
mod wrap_session_post;
pub(crate) use wrap_session_post::acp_session_set_run_timing;
mod session_drop_teardown;

#[cfg(unix)]
#[path = "hostile_orphan_test_util.rs"]
pub mod hostile_orphan_test_util;

#[cfg(test)]
#[path = "session_tests.rs"]
mod session_tests;

#[path = "wrap_agent_bundle.rs"]
mod wrap_agent_bundle;
#[path = "wrap_retry_policy.rs"]
mod wrap_retry_policy;
pub use wrap_agent_bundle::{AgentError, AgentIoOptions, AuthError};
pub(crate) use wrap_agent_bundle::*;
pub(crate) use wrap_retry_policy::*;

#[path = "agent_client_struct.rs"]
mod agent_client_struct;
pub use agent_client_struct::AgentClient;

#[path = "wrap_ops_spawn.rs"]
mod wrap_ops_spawn;
pub(crate) use wrap_ops_spawn::*;

#[path = "ops_body_kpop.rs"]
mod ops_body_kpop;
pub use ops_body_kpop::{AgentKpopMultiturnCtl, KpopFlowOnceArgs};
pub(crate) use ops_body_kpop::*;

#[path = "prompt_round_health.rs"]
mod prompt_round_health;
pub(crate) use prompt_round_health::PromptRoundHealth;

#[path = "ops_body_kpop_mt.rs"]
mod ops_body_kpop_mt;
pub(crate) use ops_body_kpop_mt::*;

#[path = "client_impl_helpers.rs"]
mod client_impl_helpers;
#[path = "client_impl_session.rs"]
mod client_impl_session;
#[path = "client_impl_prompt_dispatch.rs"]
mod client_impl_prompt_dispatch;
#[path = "client_impl_prompt_retry.rs"]
pub(crate) mod client_impl_prompt_retry;
#[path = "client_impl_prompt.rs"]
mod client_impl_prompt;
#[path = "client_impl_flow.rs"]
mod client_impl_flow;
pub(crate) use client_impl_helpers::*;
pub(crate) use client_impl_prompt_dispatch::*;

#[cfg(test)]
#[path = "ops_inline_tests.rs"]
mod ops_inline_tests;
#[cfg(test)]
#[path = "tee_strip_tests.rs"]
mod tee_strip_tests;

#[doc(hidden)]
pub mod test_captive_session;

#[cfg(test)]
mod kiss_coverage;

#[cfg(test)]
#[path = "kiss_coverage_b.rs"]
mod kiss_coverage_b;

#[cfg(test)] #[path = "inc_kiss_coverage.rs"] mod inc_kiss_coverage;
#[cfg(test)] #[path = "session_tests_kiss_cov.rs"] mod session_tests_kiss_cov;
#[cfg(test)]
pub(crate) mod spawn_test_args;
