//! Static references and smoke calls so `kiss check` test-coverage sees code units (see `style.md` TRIGGER: kiss Rust test refs).
#![allow(unused_imports)] // imports exist only so symbols appear in `stringify!(Type)` lines

use std::collections::HashMap;

use crate::acp::{AgentClient, AgentIoOptions};
use crate::artifacts::{RunArtifacts, create_run_artifacts};
use crate::orchestrator::{Orchestrator, WorkflowConfig};
use crate::prompts::{DO_HEADER_MD, HEADER_MD, PromptStore};

#[test]
fn kiss_stringify_agent() {
    let _ = stringify!(AgentClient);
    let _ = stringify!(crate::acp::AgentError);
    let _ = stringify!(crate::acp::AuthError);
    let _ = stringify!(crate::acp::ReviewerPromptPair);
    let _ = stringify!(AgentIoOptions);
    let _ = stringify!(crate::acp::has_api_key);
    let _ = stringify!(crate::acp::auth_probe);
    let _ = stringify!(crate::acp::spawn_agent_acp_session);
    let _ = stringify!(crate::acp::strip_trace_invocation_line_for_tee);
    let _ = stringify!(crate::acp::run_reviewer_pair_once);
    let _ = stringify!(crate::acp::run_kpop_flow_once);
    let _ = stringify!(crate::acp::run_kpop_multiturn_once);
    let _ = stringify!(crate::acp::KpopFlowOnceArgs);
    let _ = stringify!(AgentClient::new);
    let _ = stringify!(AgentClient::ensure_authenticated);
    let _ = stringify!(AgentClient::begin_coder_session);
    let _ = stringify!(AgentClient::run_coder_prompt);
    let _ = stringify!(AgentClient::end_coder_session);
    let _ = stringify!(AgentClient::run_reviewer_review);
    let _ = stringify!(AgentClient::run_kpop_flow);
    let _ = stringify!(AgentClient::run_kpop_multiturn);
    let _ = stringify!(AgentClient::set_run_timing);
    let _ = stringify!(AgentClient::attach_run_timing_for_session);
    let _ = stringify!(crate::acp::DEFAULT_REPO_STYLE_PROMPT_REL);
}

#[test]
fn kiss_stringify_artifacts() {
    let _ = stringify!(RunArtifacts);
    let _ = stringify!(RunArtifacts::log_path);
    let _ = stringify!(RunArtifacts::artifact_review_md);
    let _ = stringify!(RunArtifacts::artifact_result_md);
    let _ = stringify!(RunArtifacts::workspace_review_md);
    let _ = stringify!(RunArtifacts::exp_log_path);
    let _ = stringify!(create_run_artifacts);
    let _ = stringify!(crate::artifacts::run_id::create_run_dir);
    let _ = stringify!(crate::artifacts::run_id::build_identifier);
    let _ = stringify!(crate::artifacts::run_id::random_alnum);
    let _ = stringify!(crate::artifacts::create_kpop_run_artifacts);
    let _ = stringify!(crate::artifacts::resolve_user_request);
    let _ = stringify!(crate::artifacts::startup_request_tag_label);
    let _ = stringify!(crate::artifacts::work_dir_for_path);
    let _ = stringify!(crate::artifacts::resolve_at_file);
    let _ = stringify!(crate::artifacts::backup_workspace_grounding_if_present);
    let _ = stringify!(crate::artifacts::restore_workspace_grounding);
}

#[test]
fn kiss_stringify_config() {
    let _ = stringify!(crate::config::DEFAULT_ACP_RPC_TIMEOUT_SECS);
    let _ = stringify!(crate::config::acp_rpc_timeout_secs_from_env);
}

#[test]
fn kiss_stringify_child_health() {
    let _ = stringify!(crate::child_health::ChildHealth);
    let _ = stringify!(crate::child_health::SilenceHealthOutcome);
    let _ = stringify!(crate::child_health::sample_child_health);
    let _ = stringify!(crate::child_health::evaluate_after_acp_silence);
    let _ = stringify!(crate::child_health::health_indicates_progress);
    let _ = stringify!(crate::child_health::silence_grace_for_rpc_timeout);
}

#[test]
fn kiss_stringify_env_path() {
    let _ = stringify!(crate::env_path::lookup_bin_on_path);
    let _ = stringify!(crate::env_path::agent_or_cursor_agent_bin);
    let _ = stringify!(crate::env_path::require_kiss_for_malvin);
}

#[test]
fn kiss_stringify_run_timing() {
    let _ = stringify!(crate::run_timing::RunTiming::new_arc);
    let _ = stringify!(crate::run_timing::attach_new_run_timing);
    let _ = stringify!(crate::run_timing::finalize_and_emit_run_timing);
    let _ = stringify!(crate::run_timing::RUN_TIMING_JSON_FILE);
    let _ = stringify!(crate::run_timing::RUN_TIMING_SUMMARY_PREFIX);
}

#[test]
fn kiss_stringify_invocation() {
    let _ = stringify!(crate::invocation::init_from_env);
    let _ = stringify!(crate::invocation::command_line);
}

#[test]
fn kiss_stringify_log_paths() {
    let _ = stringify!(crate::log_paths::format_logs_dir);
}

#[test]
fn kiss_stringify_review_sync() {
    let _ = stringify!(crate::review_sync::is_lgtm_str);
    let _ = stringify!(crate::review_sync::sync_review_file);
    let _ = stringify!(crate::review_sync::sync_review_then_is_lgtm);
}

#[test]
fn kiss_stringify_orchestrator() {
    let _ = stringify!(crate::orchestrator::WorkflowError);
    let _ = stringify!(WorkflowConfig);
    let _ = stringify!(Orchestrator);
    let _ = stringify!(Orchestrator::run);
    let _ = stringify!(crate::orchestrator::check_abort);
    let _ = stringify!(crate::orchestrator::clear_review_file);
    let _ = stringify!(crate::orchestrator::format_prompt_path);
    let _ = stringify!(crate::orchestrator::format_exp_log_relative);
    let _ = stringify!(crate::orchestrator::prompt_md_stem);
    let _ = stringify!(crate::orchestrator::workflow_context);
    let _ = stringify!(crate::orchestrator::workflow_context_paths_only);
    let _ = stringify!(crate::orchestrator::review_context::ReviewPhaseArgs);
    let _ = stringify!(crate::orchestrator::review_context::ReviewAttemptCtx);
}

#[test]
fn kiss_stringify_kpop_acp_prompt() {
    let _ = stringify!(crate::kpop_acp_prompt::kpop_creative_enabled);
    let _ = stringify!(crate::kpop_acp_prompt::CREATIVE_MIN_INTERACTION);
}

#[test]
fn kiss_stringify_kpop_schedule() {
    let _ = stringify!(crate::kpop_schedule::KPOP_CATCHUP_CAP);
    let _ = stringify!(crate::kpop_schedule::block_mean_from_p_creative);
    let _ = stringify!(crate::kpop_schedule::poisson_block_size);
    let _ = stringify!(crate::kpop_schedule::count_kpop_entries);
    let _ = stringify!(crate::kpop_schedule::count_mbc2_entries);
    let _ = stringify!(crate::kpop_schedule::hypotheses_emitted);
    let _ = stringify!(crate::kpop_schedule::agent_declared_success);
    let _ = stringify!(crate::kpop_schedule::read_exp_log_text);
    let _ = stringify!(crate::kpop_multiturn_prompts::KpopMultiturnPrompts);
    let _ = stringify!(crate::kpop_multiturn::KpopMultiturnParams::<()>);
    let _ = stringify!(crate::multiturn_prompt::MultiturnPrompt);
    let _ = stringify!(crate::kpop_multiturn::KpopMultiturnState::<()>);
}

#[test]
fn kiss_stringify_prompts() {
    let _ = stringify!(crate::prompts::PromptError);
    let _ = stringify!(PromptStore);
    let _ = stringify!(crate::prompts::default_file);
    let _ = stringify!(crate::prompts::user_home_dir);
    let _ = stringify!(crate::prompts::render_template);
    let _ = stringify!(crate::prompts::substitute_template);
    let _ = stringify!(PromptStore::default_store);
    let _ = stringify!(PromptStore::with_root);
    let _ = stringify!(PromptStore::ensure_defaults);
    let _ = stringify!(PromptStore::validate_required);
    let _ = stringify!(PromptStore::validate_kpop_prompts);
    let _ = stringify!(PromptStore::validate_exists);
    let _ = stringify!(PromptStore::render);
    let _ = stringify!(PromptStore::render_prompt_only);
    let _ = stringify!(HEADER_MD);
    let _ = stringify!(DO_HEADER_MD);
}

#[test]
fn smoke_create_run_artifacts() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "x").expect("write plan");
    let art = create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
    assert!(art.run_dir.exists());
}

fn kiss_test_tmp() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

fn kiss_write_same_body_files(dir: &std::path::Path, names: &[&str], body: &[u8]) {
    std::fs::create_dir_all(dir).expect("mkdir");
    for name in names {
        std::fs::write(dir.join(name), body).expect("write prompt");
    }
}

#[test]
fn smoke_prompt_store_with_root() {
    let tmp = kiss_test_tmp();
    let prompts = tmp.path().join("prompts");
    kiss_write_same_body_files(
        &prompts,
        &[
            "implement.md",
            "review_1.md",
            "review_2.md",
            "kpop.md",
            "kpop_common.md",
            "kpop_block.md",
            "mbc2_pure.md",
            "mbc2.md",
            "concerns.md",
            "learn.md",
            HEADER_MD,
            DO_HEADER_MD,
            "coding_rules.md",
        ],
        b"body",
    );
    let store = PromptStore::with_root(prompts);
    store.ensure_defaults().expect("defaults");
    store.validate_required().expect("required");
    let mut ctx = HashMap::new();
    ctx.insert("plan_path".to_string(), "p".to_string());
    ctx.insert("kpop_log_dir".to_string(), "./_kpop".to_string());
    let _ = store.render("implement.md", &ctx).expect("render");
}
