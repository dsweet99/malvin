//! Malvin: implementation and review workflow driven by Cursor **`agent acp`** (ACP).
// Transitive deps (e.g. `rand` / `tempfile`) pull duplicate crate versions; `clippy::cargo` flags it.
#![allow(clippy::multiple_crate_versions)]

pub mod acp;
pub use acp::{
    AcpSession, AcpSpawnArgs, AgentClient, AgentError, AgentIoOptions, AuthError,
    CoderPromptOptions, KpopFlowOnceArgs, ReviewerPromptPair,
};

/// Compatibility shim for code that imported `malvin::agent` before the `acp`-centric layout.
#[deprecated(note = "use crate-root re-exports or `malvin::acp` instead")]
pub mod agent {
    pub use crate::{
        AcpSession, AcpSpawnArgs, AgentClient, AgentError, AgentIoOptions, AuthError,
        ReviewerPromptPair,
    };

    /// Legacy `malvin::agent::pair` path (`ReviewerPromptPair` and related).
    pub mod pair {
        pub use crate::ReviewerPromptPair;
    }
}

pub mod artifacts;
mod child_health;
pub mod config;
mod kpop_acp_prompt;
pub use kpop_acp_prompt::kpop_creative_enabled;
pub mod env_path;
pub mod invocation;
pub mod log_paths;
pub mod orchestrator;
pub mod output;
pub mod prompts;
mod review_sync;
pub mod run_timing;

#[cfg(test)]
mod coverage_kiss;

#[cfg(test)]
mod orchestrator_tests;

#[cfg(test)]
pub mod test_utils;
