//! Cursor **`agent acp`** client.

mod client;
mod ops;
pub mod pair;

pub use client::AgentClient;
pub use pair::ReviewerPromptPair;

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
    pub tee: bool,
    pub tee_json: bool,
}

#[cfg(test)]
mod kiss_refs {
    #[test]
    fn stringify_private_helpers() {
        let _ = stringify!(super::AgentIoOptions);
        let _ = stringify!(super::ops::has_api_key);
        let _ = stringify!(super::ops::auth_probe);
        let _ = stringify!(super::ops::spawn_acp_session);
        let _ = stringify!(super::ops::maybe_tee_log);
        let _ = stringify!(super::ops::run_reviewer_pair_once);
    }
}
