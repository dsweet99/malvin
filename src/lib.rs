//! Malvin: implementation and review workflow driven by the Cursor SDK (`cursor:`) or Prime SDK (`prime:`).
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
mod workflow_name_aliases;
pub use workflow_name_aliases::{
    canonical_workflow_name, resolve_session_log_path, resolve_workspace_malvin_config_path,
    WORKSPACE_CONFIG_PATHS,
};
/// Shared LLM completion types used by the local engine / Prime sidecar.
pub mod llm_transport;
/// Agent interface (malvin → cursor-agent / Prime).
pub mod agent;
/// In-process llama.cpp backend formerly the `malvin-llama` workspace crate.
pub mod malvin_llama;
mod gate_loop_session;
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
mod parent_death_signal;
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
pub mod herdr;
mod active_agent_heartbeat;
pub use active_agent_heartbeat::active_agent_heartbeat_stats;
pub use user_home::user_home_dir;
pub mod tool_summary;
mod deferred_log;
mod cursor_store;
pub use cursor_store::store_db_contains_substring;
pub mod agent_backend;
pub mod bridge_protocol;
pub mod bridge_sdk;
pub mod cursor_sdk;
pub mod prime_sdk;
#[cfg(test)]
pub(crate) mod sdk_bridge_build;
pub mod acp;
pub mod ansi_strip;
pub use acp::{AgentError, AgentIoOptions, AuthError, CoderPromptOptions};
#[cfg(unix)]
pub use acp::{snapshot_pids, terminate_agent_process_group};
pub use ansi_strip::strip_ansi_escapes;
pub use artifacts::startup_request_tag_label;
pub use artifacts::{
    MalvinChecksBackup, RunArtifacts, SessionDotfileBackups,
    backup_workspace_malvin_checks_if_present, create_run_artifacts_from_text,
    restore_workspace_session_dotfiles,
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
pub mod artifacts;
mod child_health;
mod test_poll;
pub use test_poll::{
    test_post_teardown_poll_interval, test_post_teardown_wait_budget, test_wait_until_async,
};
pub mod config;
pub mod local_llm;
pub mod model_id;
mod kpop_turn_prompts;
pub use kpop_turn_prompts::KpopTurnPrompts;
pub mod kpop_progression;
pub mod support_paths;
pub use support_paths::{
    agent_or_cursor_agent_bin, command_line, format_logs_dir, init_from_env, lookup_bin_on_path,
};
pub mod sdk_drain_timeout;
pub mod workflow_context;
pub mod orchestrator;
pub use orchestrator::check_abort;
pub use workflow_context::{
    format_malvin_command, format_prompt_path, workflow_context_paths_only, PromptModelOpts,
};
#[cfg(test)]
pub use workflow_context::workflow_context;
pub mod observability;
pub mod kpop_log_protocol;
pub mod acp_trace_impersonation;
pub mod coder_prompt_phase;
pub mod nested_budget_scopes;
pub mod prompt_stratification;
pub mod reliability_tier;
pub mod session_sandbox_policy;
pub mod output;
pub mod prompts;
pub mod repo_gates;
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
#[path = "cli/do_flow.rs"]
pub mod do_flow;
#[path = "cli/inspire_flow.rs"]
pub mod inspire_flow;
#[path = "cli/router_flow.rs"]
pub mod router_flow;
pub mod kpop_engine;
#[path = "cli/mod.rs"]
pub mod cli;
#[cfg(test)]
#[path = "lib_test_modules.rs"]
mod lib_test_modules;
#[cfg(test)] mod acp_tests;
#[cfg(test)] mod coverage_kiss;
#[cfg(test)] mod malvin_kiss_coverage;
#[cfg(test)] #[path = "malvin_kiss_coverage_b.rs"] mod malvin_kiss_coverage_b;
#[cfg(test)] mod agent_phase_kiss_cov;
#[cfg(test)] #[path = "workspace_paths_tests.rs"] mod workspace_paths_tests;
#[cfg(all(test, unix))] mod test_stderr_capture;
#[cfg(test)] mod malvin_test_seed;
#[cfg(test)] pub use malvin_test_seed::{seed_malvin_checks, seed_malvin_config};
#[cfg(test)] pub mod test_utils;
#[cfg(test)] pub mod flow_prompt_join_test_helpers;
#[cfg(test)] pub mod test_agent_client;
