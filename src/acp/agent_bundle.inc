// Agent client implementation (included into `crate::acp` so `kiss` dependency depth stays ≤2).

use std::path::PathBuf;
use std::process::Command as StdCommand;
use std::time::Duration;

use crate::review_sync::{is_lgtm, sync_review_file};

/// Recoverable agent failure.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct AgentError(pub String);

/// Missing Cursor authentication.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct AuthError(pub String);

/// CLI flags that map to subprocess / logging behavior (grouped for `kiss` boolean-parameter limits).
#[derive(Debug, Clone, Copy)]
pub struct AgentIoOptions {
    pub force: bool,
    /// When false, tee log output to stdout (default). Set when user passes `--no-tee`.
    pub no_tee: bool,
}

include!("tee_strip_body.inc");
include!("pair.inc");
include!("agent_client_struct.inc");
include!("ops_body.inc");
include!("client_impl.inc");

#[cfg(test)]
include!("tee_strip_tests.inc");

#[cfg(test)]
mod ops_resolve_bin_tests {
    #![allow(unsafe_code)]
    use std::path::Path;

    use super::resolve_agent_bin;

    include!("ops_inline_tests.inc");
}

#[test]
fn stringify_private_helpers() {
    let _ = stringify!(AgentIoOptions);
    let _ = stringify!(has_api_key);
    let _ = stringify!(auth_probe);
    let _ = stringify!(spawn_agent_acp_session);
    let _ = stringify!(maybe_tee_log);
    let _ = stringify!(strip_trace_invocation_line_for_tee);
    let _ = stringify!(run_reviewer_pair_once);
    let _ = stringify!(run_kpop_flow_once);
    let _ = stringify!(KpopFlowOnceArgs);
}
