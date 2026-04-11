//! Malvin: implementation and review workflow driven by Cursor **`agent acp`** (ACP).

pub mod acp;
pub use acp::{
    AgentClient, AgentError, AgentIoOptions, AuthError, AcpSession, AcpSpawnArgs, ReviewerPromptPair,
};

/// Compatibility shim for code that imported `malvin::agent` before the `acp`-centric layout.
#[deprecated(note = "use crate-root re-exports or `malvin::acp` instead")]
pub mod agent {
    pub use crate::{
        AgentClient, AgentError, AgentIoOptions, AuthError, AcpSession, AcpSpawnArgs, ReviewerPromptPair,
    };

    /// Legacy `malvin::agent::pair` path (`ReviewerPromptPair` and related).
    pub mod pair {
        pub use crate::ReviewerPromptPair;
    }
}

pub mod artifacts;
pub mod config;
pub mod invocation;
pub mod log_paths;
pub mod orchestrator;
pub mod prompts;
mod review_sync;

#[cfg(test)]
mod coverage_kiss;

#[cfg(test)]
mod orchestrator_tests;

#[cfg(test)]
pub mod test_utils;
