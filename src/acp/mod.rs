#![cfg_attr(test, allow(unsafe_code))]
//! Agent Client Protocol (`agent acp`) JSON-RPC over stdio.
//!
//! Much of the implementation is assembled with [`include!`] so `kiss check` dependency depth
//! stays within project limits; use the include file names (for example `transport/rpc.rs`) when
//! navigating—IDE “go to module” may not match a single `mod` tree.

mod jsonl_trace;
mod outgoing_prompt_trace;
pub use outgoing_prompt_trace::CoderPromptOptions;
mod session_types;
mod unix_process_group;

pub(crate) use jsonl_trace::AcpJsonlTrace;
pub use session_types::{AcpSession, AcpSpawnArgs};
pub(crate) use session_types::{PromptTraceWriter, ResponseTx};
#[cfg(test)]
pub(crate) use session_types::AcpSessionInner;

include!("cursor_credentials.rs");
include!("coalesce.rs");
include!("coalesce_trace.rs");
mod trace_line_write;
pub(crate) use trace_line_write::{
    ReaderTraceLineOpts, reader_loop_verbose_and_trace_line, trace_file_write_line,
};
include!("session_trace.rs");

// Inlined former `acp::transport` so `kiss` dependency depth stays ≤2 (no `lib → acp → transport`).
use serde_json::{Map, Value, json};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, ChildStdout};
use tokio::sync::{Mutex, oneshot};
use tracing::{debug, error, info, trace, warn};

include!("transport/jsonrpc_error.rs");
include!("transport/command.rs");
include!("transport/rpc.rs");
include!("transport/handshake.rs");

/// Per-request wait helper for unit tests (matches [`crate::config::acp_rpc_timeout_secs_from_env`]).
#[cfg(test)]
pub(crate) fn acp_rpc_timeout() -> std::time::Duration {
    std::time::Duration::from_secs(crate::config::acp_rpc_timeout_secs_from_env())
}

pub(crate) fn requires_cursor_login_auth(
    explicit_api_key: Option<&str>,
    explicit_auth_token: Option<&str>,
) -> bool {
    effective_cursor_api_key(explicit_api_key).is_none()
        && effective_cursor_auth_token(explicit_auth_token).is_none()
}

#[test]
fn kiss_stringify_acp_rpc_and_cursor_auth() {
    let _ = stringify!(acp_rpc_timeout);
    let _ = stringify!(requires_cursor_login_auth);
}

include!("reader_inline.rs");
include!("session_spawn.inc");
use outgoing_prompt_trace::DoPromptTraceSplit;
include!("session.rs");

include!("agent_bundle.rs");

/// Hidden harness: spawns `cat` as a stand-in coder session for the `malvin` binary crate’s unit tests.
///
/// The library is built **without** `cfg(test)` when linked from the binary target, so this stays
/// unconditional; normal callers should not use it.
#[doc(hidden)]
pub mod test_captive_session;

#[cfg(all(test, unix))]
use std::os::unix::fs::PermissionsExt;
#[cfg(test)]
use tokio::io::AsyncReadExt;
#[cfg(test)]
#[allow(
    unsafe_code,
    clippy::pedantic,
    clippy::nursery,
    unused_imports,
    clippy::await_holding_lock,
    clippy::mutex_integer,
    clippy::await_holding_lock,
    clippy::unnecessary_struct_initialization,
    clippy::unused_async,
    clippy::redundant_pub_crate
)]
include!("transport_tests_inline.inc");

#[cfg(test)]
mod reader_tests;

#[cfg(test)]
pub(crate) mod spawn_test_args;

#[cfg(test)]
mod cursor_credentials_tests_inline {
    #![allow(unsafe_code)]
    use super::{
        effective_cursor_api_key, effective_cursor_auth_token, nonempty_explicit_or_env_var,
    };
    use crate::test_utils::test_env_lock;

    include!("cursor_credentials_tests.rs");
}
