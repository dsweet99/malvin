//! Malvin: implementation and review workflow driven by Cursor **`agent acp`** (ACP).

pub mod acp;
pub mod agent;
pub use agent::AgentIoOptions;
pub mod artifacts;
pub mod config;
pub mod orchestrator;
pub mod prompts;
mod review_sync;

#[cfg(test)]
mod coverage_kiss;

#[cfg(test)]
pub mod test_utils;
