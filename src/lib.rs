//! Malvin: implementation and review workflow driven by Cursor **`agent acp`** (ACP).
#![cfg_attr(
    test,
    allow(
        clippy::mutex_integer,
        clippy::await_holding_lock,
        clippy::unnecessary_struct_initialization,
        dead_code,
        clippy::use_self
    )
)]
#![allow(
    clippy::multiple_crate_versions,
    unused_attributes,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::needless_pass_by_value,
    clippy::redundant_pub_crate,
    clippy::unused_async,
    clippy::implicit_hasher,
    clippy::unnecessary_lazy_evaluations,
    clippy::redundant_clone,
    clippy::needless_borrow,
    clippy::elidable_lifetime_names,
    clippy::match_same_arms,
    clippy::ptr_arg,
    clippy::unused_self,
    clippy::assigning_clones,
    clippy::no_effect_underscore_binding,
    clippy::implicit_clone,
    clippy::single_match,
    clippy::needless_pass_by_ref_mut
)]

mod alnum_id;
mod learn_gate;
pub mod session_dotfile_backup;
pub mod run_id;
mod malvin_constants;
mod tracing_init;
mod user_home;
pub use learn_gate::{DEFAULT_LEARN_MIN_ELAPSED_MS, should_run_learn_check};
pub(crate) mod time_format;
pub use user_home::user_home_dir;
pub mod acp;
pub mod ansi_strip;
pub use acp::{
    AgentClient, AgentError, AgentIoOptions, AgentKpopMultiturnCtl, AuthError, AcpSession,
    AcpSpawnArgs, CoderPromptOptions, KpopFlowOnceArgs, ReviewerPromptPair,
};
pub use session_dotfile_backup::KissConfigBackup;
pub use artifacts::{
    MalvinChecksBackup, RunArtifacts, SessionDotfileBackups,
    backup_workspace_kissconfig_if_present, backup_workspace_kissignore_if_present,
    backup_workspace_malvin_checks_if_present, create_run_artifacts_from_text,
    restore_workspace_session_dotfiles,
};
pub use ansi_strip::strip_ansi_escapes;
pub use artifacts::{create_kpop_run_artifacts, create_run_artifacts, resolve_user_request};
pub use artifacts::startup_request_tag_label;
pub use prompts::DO_HEADER_MD;
pub use config::DEFAULT_CLI_MODEL;
pub use kpop_progression::agent_declared_success;
pub use output::{
    MALVIN_WHO, format_line, format_log_tag_inner, init_stdout_style, print_stderr_line,
    print_stdout_line, print_stdout_text,
};
pub use prompts::{HEADER_MD, PromptError, PromptStore, merged_coding_rules};
pub use run_timing::{
    RunTiming, TimingPhase, finalize_and_emit_run_timing, finalize_run_timing_json_only,
};

pub(crate) mod acp_memory_containment;
pub mod artifacts;
mod child_health;
pub mod config;
mod kpop_acp_prompt;
pub use kpop_acp_prompt::kpop_creative_enabled;
mod kpop_test_stubs;
mod kpop_turn_prompts;
pub use kpop_test_stubs::{CaptureWants as KpopCaptureWants, EchoPrompts as KpopEchoPrompts, MtStubPrompts};
pub use kpop_turn_prompts::KpopTurnPrompts;
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
pub use orchestrator::{
    Orchestrator, ReviewTwoPromptSession, ReviewWriteInnerOutcome, WorkflowConfig, WorkflowError,
    REVIEW_WRITE_MISSING_ARTIFACT_MSG, REVIEW_WRITE_MISSING_ARTIFACT_RETRY_MSG, check_abort,
    fail_on_abort_for_artifacts, format_prompt_path, run_reviewers_spawn_then_review_write,
    workflow_context, workflow_context_paths_only,
};
pub mod output;
pub mod prompts;
pub mod repo_gates;
pub mod stdout_log_path;
pub mod review_sync;
pub mod run_timing;

pub mod acp_post_run {
    pub use crate::run_timing::acp_post_run::*;
}

#[path = "cli/repo_checks/mod.rs"]
pub mod repo_checks;

#[path = "cli/source_detect.rs"]
pub mod source_detect;

#[path = "cli/init_cmd.rs"]
pub mod init_cmd;

#[path = "cli/do_flow.rs"]
pub mod do_flow;

#[path = "cli/plan_flow/mod.rs"]
pub mod plan_flow;

#[path = "cli/mod.rs"]
pub mod cli;


#[cfg(test)]
#[path = "acp/test_unix_bin.rs"]
pub mod acp_test_unix_bin;

#[cfg(test)]
#[path = "acp_session_tests/mod.rs"]
pub(crate) mod acp_session_unit_tests;

#[cfg(test)]
mod coverage_kiss;

#[cfg(test)]
mod coverage_kiss_agent;

#[cfg(test)]
mod orchestrator_tests;

#[cfg(test)]
mod orchestrator_check_plan_tests;

#[cfg(test)]
mod malvin_kiss_coverage;

#[cfg(test)]
#[allow(unsafe_code)]
mod review_prep_regression;

#[cfg(all(test, unix))]
mod test_stderr_capture;

#[cfg(test)]
pub mod test_utils;
