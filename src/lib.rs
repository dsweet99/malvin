//! Malvin: implementation and review workflow driven by Cursor **`agent acp`** (ACP).
// Transitive deps (e.g. `rand` / `tempfile`) pull duplicate crate versions; `clippy::cargo` flags it.
#![allow(clippy::multiple_crate_versions)]

pub mod acp;
pub mod ansi_strip;
pub use acp::{
    AcpSession, AcpSpawnArgs, AgentClient, AgentError, AgentIoOptions, AuthError,
    CoderPromptOptions, KpopFlowOnceArgs, ReviewerPromptPair,
};

pub mod artifacts;
mod child_health;
pub mod config;
mod kpop_acp_prompt;
pub use kpop_acp_prompt::kpop_creative_enabled;
pub mod kpop_multiturn_prompts;
pub use kpop_multiturn_prompts::KpopMultiturnPrompts;
pub mod kpop_progression;
mod multiturn_prompt;
pub use kpop_progression::{KpopMultiturnParams, KpopMultiturnState};
pub use multiturn_prompt::MultiturnPrompt;
pub mod support_paths;
pub use support_paths::{
    agent_or_cursor_agent_bin, command_line, format_logs_dir, init_from_env, lookup_bin_on_path,
    require_kiss_for_malvin,
};
pub mod orchestrator;
pub mod output;
pub mod prompts;
pub mod repo_gates;
pub mod review_sync;
pub mod run_timing;

#[cfg(test)]
mod coverage_kiss;

#[cfg(test)]
mod coverage_kiss_agent;

#[cfg(test)]
mod orchestrator_tests;

#[cfg(test)]
mod orchestrator_check_plan_tests;

#[cfg(test)]
pub mod test_utils;
