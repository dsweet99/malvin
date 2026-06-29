//! Malvin: implementation and review workflow driven by Cursor **`agent acp`** (ACP).
#![cfg_attr(
    test,
    allow(
        clippy::mutex_integer,
        clippy::await_holding_lock,
        clippy::unnecessary_struct_initialization,
        clippy::large_stack_arrays,
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

mod log_gc;
mod log_gc_config;
mod malvin_config_file;
mod gate_loop_session;
mod sequential_requests;
mod sandbox_oom;
mod current_state;
pub mod mem_limit_config;
pub use sandbox_oom::{
    OOM_REASON_MEASUREMENT_FAIL_CLOSED, OOM_REASON_MEMORY_LIMIT, SandboxOomKillFacts,
    SandboxOomKillRecord, gate_iteration_oom_killed, record_sandbox_oom_kill,
};
pub use current_state::format_current_state;
mod acp_spawn_lock;
mod acp_spawn_sweep;
mod session_name;
pub use session_name::{
    acquire_name, acquire_session_name, assert_no_peer_name_lock, generate_auto_name,
    generate_auto_name_with, name_path, names_registry_root, parse_holder_pid, release_name,
    validate_name, SessionNameGuard,
};
pub use acp_spawn_lock::{
    acquire_acp_spawn_lock_for_slot, active_acp_lock_slot,
    assert_no_peer_acp_spawn_lock_for_slot, release_acp_spawn_lock, set_active_acp_lock_slot,
};
pub use acp_spawn_sweep::sweep_stale_acp_spawn_locks;
pub mod malvin_sandbox;
#[cfg(test)]
#[path = "malvin_sandbox_tests.rs"]
mod malvin_sandbox_tests;
pub mod process_group_rss;
mod alnum_id;
mod malvin_short_id;
pub use malvin_short_id::{
    is_valid_malvin_short_id, malvin_short_id, validate_malvin_short_id, MALVIN_SHORT_ID_LEN,
};
mod malvin_constants;
pub mod workspace_paths;
pub use workspace_paths::{
    canonical_work_dir_for_logs, find_malvin_logs_root, git_worktree_toplevel, is_malvin_workspace,
    legacy_malvin_checks_path, malvin_acp_spawn_chamber_dir, malvin_advice_path, malvin_checks_path,
    malvin_config_path, malvin_data_root, malvin_home_config_path, malvin_home_logs_root,
    malvin_logs_root, malvin_user_home_root, read_work_dir_manifest, remove_legacy_malvin_checks_file,
    resolve_malvin_checks_path, workspace_logs_hash, write_work_dir_manifest, MALVIN_ADVICE_REL,
    MALVIN_CHECKS_REL, MALVIN_CONFIG_REL, MALVIN_DIR, MALVIN_HOME_CONFIG_FILE, MALVIN_LOGS_REL,
    MALVIN_TEST_ALLOW_HOME_CONFIG_MUTATION, MALVIN_USER_HOME_DIR, WORK_DIR_MANIFEST,
};
mod terminal_palette;
mod run_id;
pub use run_id::{build_identifier, create_run_dir, RunDirOptions};
pub mod session_dotfile_backup;
mod tracing_init;
mod user_home;
pub(crate) mod time_format;
pub mod agent_phase;
mod active_agent_heartbeat;
pub use active_agent_heartbeat::active_agent_heartbeat_stats;
pub use user_home::user_home_dir;
pub mod tool_summary;
mod deferred_log;
mod cursor_store;
pub use cursor_store::store_db_contains_substring;
mod acp_test_mock_js;
pub use acp_test_mock_js::acp_mock_js;
pub mod agent_backend;
pub mod acp;
pub mod ansi_strip;
pub use acp::{
    AcpSession, AcpSpawnArgs, AgentClient, AgentError, AgentIoOptions, AgentKpopMultiturnCtl,
    AuthError, CoderPromptOptions, KpopFlowOnceArgs,
};
#[cfg(unix)]
pub use acp::{snapshot_pids, terminate_agent_process_group};
pub use ansi_strip::strip_ansi_escapes;
pub use artifacts::startup_request_tag_label;
pub use artifacts::{
    MalvinChecksBackup, RunArtifacts, SessionDotfileBackups,
    backup_workspace_kissconfig_if_present, backup_workspace_kissignore_if_present,
    backup_workspace_malvin_checks_if_present, backup_workspace_malvin_config_if_present,
    create_run_artifacts_from_text, restore_workspace_session_dotfiles,
};
pub use artifacts::{create_kpop_run_artifacts, create_run_artifacts, resolve_user_md_request};
pub use config::DEFAULT_CLI_MODEL;
pub use kpop_progression::agent_declared_success;
pub use output::{
    ERROR_WHO, MALVIN_WHO, WARNING_WHO, format_line, format_log_tag_inner, format_who_tag_prefix,
    init_stdout_style,
    print_log_error, print_log_warning, print_stderr_line, print_stdout_line, print_stdout_text,
};
pub use prompts::DO_HEADER_MD;
pub use prompts::{
    HEADER_MD, PromptError, PromptStore, malformed_brace_placeholders, render_header,
};
pub use run_timing::{
    RunTiming, TimingPhase, finalize_and_emit_run_timing, finalize_run_timing_json_only,
    print_summary_from_run_dir,
};
pub use session_dotfile_backup::KissConfigBackup;

pub mod artifacts;
mod child_health;
mod test_poll;
pub use test_poll::{
    test_post_teardown_poll_interval, test_post_teardown_wait_budget, test_wait_until_async,
};
pub mod config;
mod kpop_test_stubs;
mod kpop_turn_prompts;
pub use kpop_test_stubs::{
    CaptureWants as KpopCaptureWants, EchoPrompts as KpopEchoPrompts, MtStubPrompts,
};
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
pub mod workflow_context;
pub mod orchestrator;
pub use orchestrator::{
    Orchestrator, WorkflowConfig, WorkflowError, check_abort, fail_on_abort_for_artifacts,
};
pub use workflow_context::{format_prompt_path, workflow_context, workflow_context_paths_only};
pub mod observability;
pub mod kpop_log_protocol;
pub mod kpop_program;
pub mod kpop_soft_constraints;
pub mod acp_trace_impersonation;
pub mod fork_state;
pub mod nested_budget_scopes;
pub mod prompt_stratification;
pub mod reliability_tier;
pub mod output;
pub mod prompts;
pub mod repo_gates;
pub mod review_sync;
pub mod run_timing;
pub mod stdout_log_path;

pub mod acp_post_run {
    pub use crate::run_timing::acp_post_run::*;
}

#[path = "cli/repo_checks/mod.rs"]
pub mod repo_checks;

#[path = "cli/source_detect.rs"]
pub mod source_detect;

#[cfg(test)]
#[path = "cli/source_detect_kiss_cov_tests.rs"]
mod source_detect_kiss_cov_tests;

#[path = "cli/init_cmd.rs"]
pub mod init_cmd;

#[path = "cli/do_flow.rs"]
pub mod do_flow;

#[path = "cli/inspire_flow.rs"]
pub mod inspire_flow;

pub mod kpop_engine;

#[path = "cli/mod.rs"]
pub mod cli;

#[cfg(test)]
#[path = "lib_test_modules.rs"]
mod lib_test_modules;

#[cfg(test)]
#[path = "acp/test_unix_bin.rs"]
pub mod acp_test_unix_bin;

#[cfg(test)]
#[path = "acp_session_tests/mod.rs"]
pub(crate) mod acp_session_unit_tests;

#[cfg(test)] mod acp_tests;
#[cfg(test)] #[path = "acp_transport_tests/mod.rs"] mod acp_transport_tests;
#[cfg(test)] mod coverage_kiss;
#[cfg(test)] mod coverage_kiss_agent;
#[cfg(test)] mod orchestrator_tests;
#[cfg(test)] mod malvin_kiss_coverage;
#[cfg(test)] #[path = "acp/transport/rpc_part1_kiss_test.rs"] mod acp_rpc_part1_kiss_test;
#[cfg(test)] mod agent_phase_kiss_cov;
#[cfg(test)] #[path = "workspace_paths_tests.rs"] mod workspace_paths_tests;
#[cfg(all(test, unix))] mod test_stderr_capture;
#[cfg(test)] mod malvin_test_seed;
#[cfg(test)] pub use malvin_test_seed::{seed_malvin_checks, seed_malvin_config};
#[cfg(test)] pub mod test_utils;
#[cfg(test)] pub mod test_agent_client;
