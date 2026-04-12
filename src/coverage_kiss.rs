//! Static references and smoke calls so `kiss check` test-coverage sees code units (see `style.md` TRIGGER: kiss Rust test refs).
#![allow(unused_imports)] // imports exist only so symbols appear in `stringify!(Type)` lines

use std::collections::HashMap;

use crate::acp::{AgentClient, AgentIoOptions};
use crate::artifacts::{RunArtifacts, create_run_artifacts};
use crate::orchestrator::{Orchestrator, WorkflowConfig};
use crate::prompts::PromptStore;

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
    let _ = stringify!(crate::acp::maybe_tee_log);
    let _ = stringify!(crate::acp::strip_trace_invocation_line_for_tee);
    let _ = stringify!(crate::acp::run_reviewer_pair_once);
    let _ = stringify!(crate::acp::run_kpop_flow_once);
    let _ = stringify!(crate::acp::KpopFlowOnceArgs);
    let _ = stringify!(AgentClient::new);
    let _ = stringify!(AgentClient::ensure_authenticated);
    let _ = stringify!(AgentClient::begin_coder_session);
    let _ = stringify!(AgentClient::run_coder_prompt);
    let _ = stringify!(AgentClient::end_coder_session);
    let _ = stringify!(AgentClient::run_reviewer_review_and_kpop);
    let _ = stringify!(AgentClient::run_kpop_flow);
}

#[test]
fn kiss_stringify_artifacts() {
    let _ = stringify!(RunArtifacts);
    let _ = stringify!(RunArtifacts::log_path);
    let _ = stringify!(create_run_artifacts);
    let _ = stringify!(crate::artifacts::create_run_dir);
    let _ = stringify!(crate::artifacts::build_identifier);
    let _ = stringify!(crate::artifacts::random_alnum);
    let _ = stringify!(crate::artifacts::create_kpop_run_artifacts);
    let _ = stringify!(crate::artifacts::resolve_user_request);
    let _ = stringify!(crate::artifacts::work_dir_for_path);
    let _ = stringify!(crate::artifacts::resolve_at_file);
}

#[test]
fn kiss_stringify_config() {
    let _ = stringify!(crate::config::DEFAULT_ACP_RPC_TIMEOUT_SECS);
    let _ = stringify!(crate::config::acp_rpc_timeout_secs_from_env);
}

#[test]
fn kiss_stringify_env_path() {
    let _ = stringify!(crate::env_path::lookup_bin_on_path);
    let _ = stringify!(crate::env_path::agent_or_cursor_agent_bin);
}

#[test]
fn kiss_stringify_edit_efficiency() {
    let _ = stringify!(crate::edit_efficiency::EditEfficiencyReport);
    let _ = stringify!(crate::edit_efficiency::EditEfficiencyError);
    let _ = stringify!(crate::edit_efficiency::EditEfficiencyMeter);
    let _ = stringify!(crate::edit_efficiency::EditEfficiencyMeter::new);
    let _ = stringify!(crate::edit_efficiency::EditEfficiencyMeter::checkpoint);
    let _ = stringify!(crate::edit_efficiency::EditEfficiencyMeter::finish);
    let _ = stringify!(crate::edit_efficiency::byte_cost::byte_edit_cost);
    let _ = stringify!(crate::edit_efficiency::maybe_checkpoint);
    let _ = stringify!(crate::edit_efficiency::finish_and_write_report);
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
    let _ = stringify!(crate::review_sync::is_lgtm);
    let _ = stringify!(crate::review_sync::sync_review_file);
}

#[test]
fn kiss_stringify_orchestrator() {
    let _ = stringify!(crate::orchestrator::WorkflowError);
    let _ = stringify!(WorkflowConfig);
    let _ = stringify!(Orchestrator);
    let _ = stringify!(Orchestrator::run);
    let _ = stringify!(crate::orchestrator::clear_review_file);
    let _ = stringify!(crate::orchestrator::format_prompt_path);
    let _ = stringify!(crate::orchestrator::prompt_md_stem);
    let _ = stringify!(crate::orchestrator::workflow_context);
    let _ = stringify!(crate::orchestrator::review_context::ReviewPhaseArgs);
    let _ = stringify!(crate::orchestrator::review_context::ReviewAttemptCtx);
}

#[test]
fn kiss_stringify_kpop_acp_prompt() {
    let _ = stringify!(crate::kpop_acp_prompt::kpop_creative_enabled);
    let _ = stringify!(crate::kpop_acp_prompt::kpop_acp_user_prompt);
    let _ = stringify!(crate::kpop_acp_prompt::kpop_standalone_outbound_prompt_count);
    let _ = stringify!(crate::kpop_acp_prompt::KpopAcpPromptPick);
    let _ = stringify!(crate::kpop_acp_prompt::CREATIVE_MIN_INTERACTION);
    let _ = stringify!(crate::kpop_acp_prompt::KPOP_SESSION_PROMPT_COUNT_WHEN_P_CREATIVE);
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
            "mbc2.md",
            "concerns.md",
            "learn.md",
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

#[test]
fn smoke_orchestrator_instantiation() {
    let tmp = kiss_test_tmp();
    let prompts_dir = tmp.path().join("prompts");
    kiss_write_same_body_files(
        &prompts_dir,
        &[
            "implement.md",
            "review_1.md",
            "review_2.md",
            "kpop.md",
            "concerns.md",
        ],
        b"Hello {{ plan_path }} $kpop_log_dir",
    );
    let store = PromptStore::with_root(prompts_dir);
    let run_dir = tmp.path().join("_malvin").join("run");
    std::fs::create_dir_all(&run_dir).expect("run dir");
    let plan_path = run_dir.join("plan.md");
    std::fs::write(&plan_path, "plan").expect("plan");
    let artifacts = RunArtifacts {
        run_dir,
        plan_path,
        work_dir: tmp.path().to_path_buf(),
    };
    let mut client = AgentClient::new(
        "m".to_string(),
        AgentIoOptions {
            force: true,
            no_tee: false,
        },
    );
    let _ = Orchestrator {
        client: &mut client,
        prompts: &store,
        artifacts: &artifacts,
        config: WorkflowConfig {
            max_loops: 1,
            run_learn: false,
        },
        progress_callback: Box::new(|_: &str| {}),
    };
}
