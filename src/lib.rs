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
extern crate self as malvin;
pub mod clock;
pub use config::workflow_name_aliases::{
    WORKSPACE_CONFIG_PATHS, canonical_workflow_name, resolve_session_log_path,
    resolve_workspace_malvin_config_path,
};
pub mod agent;
pub mod backends;
mod current_state;
pub mod gate_loop_session;
pub mod test_support;
pub mod workspace;
pub use agent_process::sandbox_oom::{
    OOM_REASON_MEASUREMENT_FAIL_CLOSED, OOM_REASON_MEMORY_LIMIT, SandboxOomKillFacts,
    SandboxOomKillRecord, gate_iteration_oom_killed, record_sandbox_oom_kill,
};
pub use current_state::format_current_state;
mod session_name;
pub use agent_process::acp_spawn_lock::{
    acquire_acp_spawn_lock_for_slot, active_acp_lock_slot, assert_no_peer_acp_spawn_lock_for_slot,
    release_acp_spawn_lock, set_active_acp_lock_slot, wait_for_dir_entry_count,
};
pub use agent_process::acp_spawn_sweep::sweep_stale_acp_spawn_locks;
pub use session_name::{
    SessionNameGuard, acquire_name, acquire_session_name, assert_no_peer_name_lock,
    generate_auto_name, generate_auto_name_with, name_path, names_registry_root, parse_holder_pid,
    release_name, validate_name,
};
mod alnum_id;
mod malvin_short_id;
pub use malvin_short_id::{
    MALVIN_SHORT_ID_LEN, is_valid_malvin_short_id, malvin_short_id, validate_malvin_short_id,
};
mod malvin_constants;
pub use workspace::workspace_paths::{
    MALVIN_ADVICE_REL, MALVIN_CHECKS_REL, MALVIN_CONFIG_REL, MALVIN_DIR, MALVIN_HOME_CONFIG_FILE,
    MALVIN_LOGS_REL, MALVIN_TEST_ALLOW_HOME_CONFIG_MUTATION, MALVIN_USER_HOME_DIR,
    WORK_DIR_MANIFEST, canonical_work_dir_for_logs, find_malvin_logs_root, git_worktree_toplevel,
    is_malvin_workspace, legacy_malvin_checks_path, malvin_acp_spawn_chamber_dir,
    malvin_advice_path, malvin_checks_path, malvin_config_path, malvin_data_root,
    malvin_home_config_path, malvin_home_logs_root, malvin_logs_root, malvin_user_home_root,
    migrate_legacy_malvin_user_home, read_work_dir_manifest, remove_legacy_malvin_checks_file,
    resolve_malvin_checks_path, workspace_logs_hash, write_work_dir_manifest,
};
pub mod terminal_palette;
pub use workspace::run_id::{RunDirOptions, build_identifier, create_run_dir};
pub mod agent_phase;
pub mod herdr;
pub mod session_dotfile_backup;
pub(crate) mod time_format;
mod tracing_init;
pub use agent_process::active_agent_heartbeat::active_agent_heartbeat_stats;
pub use workspace::user_home::user_home_dir;
pub mod agent_process;
pub mod ansi_strip;
pub(crate) mod http_fetch;
pub mod local_llm;
pub mod modal_run;
mod npm_bridge_install;
pub mod pricing;
#[cfg(test)]
pub(crate) mod sdk_bridge_build;
pub mod tool_summary;
pub use agent_process::{AgentError, AgentFault, AgentIoOptions, AuthError, CoderPromptOptions};
#[cfg(unix)]
pub use agent_process::{snapshot_pids, terminate_agent_process_group};
pub use ansi_strip::strip_ansi_escapes;
pub use artifacts::startup_request_tag_label;
pub use artifacts::{
    MalvinChecksBackup, RunArtifacts, SessionDotfileBackups,
    backup_workspace_malvin_checks_if_present, create_run_artifacts_from_text,
    restore_workspace_session_dotfiles,
};
pub use artifacts::{create_run_artifacts, resolve_user_md_request};
pub use config::DEFAULT_CLI_MODEL;
pub use output::{
    ERROR_WHO, MALVIN_WHO, WARNING_WHO, format_line, format_log_tag_inner, format_who_tag_prefix,
    init_stdout_style, print_log_error, print_log_warning, print_stderr_line, print_stdout_line,
    print_stdout_text,
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
mod cli;
pub use cli::{Exit, entrypoint};
#[cfg(any(test, debug_assertions))]
pub use test_support::test_poll::{
    test_post_teardown_poll_interval, test_post_teardown_wait_budget, test_wait_until_async,
};
pub mod config;
pub use workspace::support_paths::{
    agent_or_cursor_agent_bin, command_line, format_logs_dir, init_from_env, lookup_bin_on_path,
};
pub mod command_output_timeout;
pub mod orchestrator;
pub mod sdk_drain_timeout;
pub mod workflow_context;
pub use orchestrator::check_abort;
#[cfg(test)]
pub use workflow_context::workflow_context;
pub use workflow_context::{
    PromptModelOpts, format_malvin_command, format_prompt_path, workflow_context_paths_only,
};
pub mod acp_trace_impersonation;
pub mod coder_prompt_phase;
pub mod nested_budget_scopes;
pub mod observability;
pub mod output;
pub mod prompt_stratification;
pub mod prompts;
pub mod reliability_tier;
pub mod repo_gates;
pub mod run_timing;
pub mod acp_post_run {
    pub use crate::run_timing::acp_post_run::*;
}
#[cfg(test)]
mod acp_tests;
#[cfg(test)]
mod agent_phase_kiss_cov;
#[cfg(test)]
mod coverage_kiss;
#[cfg(test)]
#[path = "lib_test_modules.rs"]
mod lib_test_modules;
#[cfg(test)]
pub use test_support::malvin_test_seed::{seed_malvin_checks, seed_malvin_config};
#[cfg(test)]
pub mod flow_prompt_join_test_helpers;
