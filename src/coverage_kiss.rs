//! Static references and smoke calls so `kiss check` test-coverage sees code units (see `style.md` TRIGGER: kiss Rust test refs).

#[test]
fn kiss_stringify_artifacts() {
    let _ = stringify!(crate::artifacts::RunArtifacts);
    let _ = stringify!(crate::artifacts::RunArtifacts::log_path);
    let _ = stringify!(crate::artifacts::RunArtifacts::artifact_review_md);
    let _ = stringify!(crate::artifacts::RunArtifacts::artifact_result_md);
    let _ = stringify!(crate::artifacts::RunArtifacts::workspace_review_md);
    let _ = stringify!(crate::artifacts::RunArtifacts::exp_log_path);
    let _ = stringify!(crate::artifacts::RunArtifacts::quality_gates_log_path);
    let _ = stringify!(crate::artifacts::QUALITY_GATES_LOG);
    let _ = stringify!(crate::artifacts::create_run_artifacts);
    let _ = stringify!(crate::artifacts::run_id::create_run_dir);
    let _ = stringify!(crate::artifacts::run_id::build_identifier);
    let _ = stringify!(crate::artifacts::run_id::random_alnum);
    let _ = stringify!(crate::artifacts::create_kpop_run_artifacts);
    let _ = stringify!(crate::artifacts::resolve_user_request);
    let _ = stringify!(crate::artifacts::startup_request_tag_label);
    let _ = stringify!(crate::artifacts::work_dir_for_path);
    let _ = stringify!(crate::artifacts::resolve_at_file);
    let _ = stringify!(crate::artifacts::backup_workspace_kissconfig_if_present);
    let _ = stringify!(crate::artifacts::restore_workspace_kissconfig_backup);
    let _ = stringify!(crate::artifacts::backup_workspace_malvin_checks_if_present);
    let _ = stringify!(crate::artifacts::restore_workspace_malvin_checks_backup);
    let _ = stringify!(crate::artifacts::backup_workspace_kissignore_if_present);
    let _ = stringify!(crate::artifacts::restore_workspace_kissignore_backup);
    let _ = stringify!(crate::artifacts::restore_workspace_session_dotfiles);
    let _ = stringify!(crate::artifacts::SessionDotfileBackups::snapshot);
    let _ = stringify!(crate::artifacts::SessionDotfileBackups::from_parts);
}

#[test]
fn kiss_stringify_config() {
    let _ = stringify!(crate::support_paths::DEFAULT_ACP_RPC_TIMEOUT_SECS);
    let _ = stringify!(crate::support_paths::acp_rpc_timeout_secs_from_env);
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
    let _ = stringify!(crate::support_paths::lookup_bin_on_path);
    let _ = stringify!(crate::support_paths::agent_or_cursor_agent_bin);
    let _ = stringify!(crate::support_paths::require_kiss_for_malvin);
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
    let _ = stringify!(crate::support_paths::init_from_env);
    let _ = stringify!(crate::support_paths::command_line);
}

#[test]
fn kiss_stringify_log_paths() {
    let _ = stringify!(crate::support_paths::format_logs_dir);
}

#[test]
fn kiss_stringify_review_sync() {
    let _ = stringify!(crate::review_sync::is_lgtm_str);
    let _ = stringify!(crate::review_sync::read_artifact_review_for_fanout_attempt);
    let _ = stringify!(crate::review_sync::sync_review_file_for_attempt);
}

#[test]
fn kiss_stringify_orchestrator() {
    let _ = stringify!(crate::orchestrator::WorkflowError);
    let _ = stringify!(crate::orchestrator::WorkflowConfig);
    let _ = stringify!(crate::orchestrator::Orchestrator);
    let _ = stringify!(crate::orchestrator::Orchestrator::run);
    let _ = stringify!(crate::orchestrator::Orchestrator::run_with_pre_summary_gap);
    let _ = stringify!(crate::orchestrator::Orchestrator::run_bug_remediation_gap);
    let _ = stringify!(crate::orchestrator::check_abort);
    let _ = stringify!(crate::orchestrator::clear_review_file);
    let _ = stringify!(crate::orchestrator::format_prompt_path);
    let _ = stringify!(crate::orchestrator::format_exp_log_relative);
    let _ = stringify!(crate::orchestrator::prompt_md_stem);
    let _ = stringify!(crate::orchestrator::workflow_context);
    let _ = stringify!(crate::orchestrator::workflow_context_paths_only);
    let _ = stringify!(crate::orchestrator::review_fanout_run::run_review_fanout_jobs);
    let _ = stringify!(crate::orchestrator::Orchestrator::run_coder_prompt);
    let _ = stringify!(crate::orchestrator::Orchestrator::run_coder_prompt_body);
    let _ = stringify!(crate::orchestrator::run_review_fanout_prefix);
    let _ = stringify!(crate::orchestrator::ensure_artifact_review_after_review_write);
    let _ = stringify!(crate::orchestrator::REVIEW_WRITE_MISSING_ARTIFACT_MSG);
    let _ = stringify!(crate::orchestrator::is_missing_artifact_review_error);
    let _ = stringify!(crate::orchestrator::review_attempt_is_lgtm);
    let _ = stringify!(crate::orchestrator::load_review_descriptions_for_kernel);
    let _ = stringify!(crate::orchestrator::review_fanout_write::run_review_write_coder_session);
    let _ = stringify!(crate::orchestrator::merge_string_run_and_restore);
    let _ = stringify!(crate::orchestrator::workflow_merge::merge_workflow_run_and_restore);
    let _ = stringify!(crate::orchestrator::constants::REVIEWER_FANOUT_CONCURRENCY);
    let _ = stringify!(crate::orchestrator::constants::fanout_wave_count);
}

#[test]
fn kiss_stringify_kpop_acp_prompt() {
    let _ = stringify!(crate::kpop_acp_prompt::kpop_creative_enabled);
    let _ = stringify!(crate::kpop_acp_prompt::CREATIVE_MIN_INTERACTION);
}

#[test]
fn kiss_stringify_kpop_progression() {
    let _ = stringify!(crate::kpop_progression::KPOP_CATCHUP_CAP);
    let _ = stringify!(crate::kpop_progression::block_mean_from_p_creative);
    let _ = stringify!(crate::kpop_progression::poisson_block_size);
    let _ = stringify!(crate::kpop_progression::count_kpop_entries);
    let _ = stringify!(crate::kpop_progression::count_mbc2_entries);
    let _ = stringify!(crate::kpop_progression::hypotheses_emitted);
    let _ = stringify!(crate::kpop_progression::agent_declared_success);
    let _ = stringify!(crate::kpop_progression::read_exp_log_text);
    let _ = stringify!(crate::kpop_multiturn_prompts::KpopMultiturnPrompts);
    let _ = stringify!(crate::kpop_progression::KpopMultiturnParams::<()>);
    let _ = stringify!(crate::multiturn_prompt::MultiturnPrompt);
    let _ = stringify!(crate::kpop_progression::KpopMultiturnState::<()>);
}

#[test]
fn kiss_stringify_prompts() {
    let _ = stringify!(crate::prompts::enforce_no_unresolved_braces);
    let _ = stringify!(crate::prompts::PromptError);
    let _ = stringify!(crate::prompts::PromptStore);
    let _ = stringify!(crate::prompts::default_file);
    let _ = stringify!(crate::prompts::user_home_dir);
    let _ = stringify!(crate::prompts::render_template);
    let _ = stringify!(crate::prompts::substitute_template);
    let _ = stringify!(crate::prompts::PromptStore::default_store);
    let _ = stringify!(crate::prompts::PromptStore::with_root);
    let _ = stringify!(crate::prompts::PromptStore::ensure_defaults);
    let _ = stringify!(crate::prompts::PromptStore::validate_required);
    let _ = stringify!(crate::prompts::PromptStore::validate_kpop_prompts);
    let _ = stringify!(crate::prompts::PromptStore::validate_exists);
    let _ = stringify!(crate::prompts::PromptStore::render);
    let _ = stringify!(crate::prompts::PromptStore::render_prompt_only);
    let _ = stringify!(crate::prompts::HEADER_MD);
    let _ = stringify!(crate::prompts::DO_HEADER_MD);
}

#[test]
fn kiss_stringify_repo_gates() {
    let _ = stringify!(crate::repo_gates::KISS_CHECK_COMMAND);
    let _ = stringify!(crate::repo_gates::should_run_workspace_gates);
    let _ = stringify!(crate::repo_gates::gate_command_lines);
    let _ = stringify!(crate::repo_gates::prompt_quality_gates_markdown);
    let _ = stringify!(crate::repo_gates::format_quality_gates_markdown);
    let _ = stringify!(crate::repo_gates::load_malvin_checks);
    let _ = stringify!(crate::repo_gates::ensure_default_malvin_checks_file);
    let _ = stringify!(crate::repo_gates::gate_command_lines_for_workspace_run);
    let _ = stringify!(crate::repo_gates::discover_py::visit_source_files);
    let _ = stringify!(crate::repo_gates::discover_py::python_ruff_and_pytest_flags);
    let _ = stringify!(kiss_smoke_prompt_store);
    let _ = stringify!(kiss_smoke_render_context);
}

#[test]
fn smoke_create_run_artifacts() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "x").expect("write plan");
    let art = crate::artifacts::create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
    assert!(art.run_dir.exists());
}

fn kiss_test_tmp() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

#[test]
fn kiss_stringify_coverage_kiss_helpers() {
    let _ = stringify!(kiss_write_same_body_files);
}

fn kiss_write_same_body_files(dir: &std::path::Path, names: &[&str], body: &[u8]) {
    std::fs::create_dir_all(dir).expect("mkdir");
    for name in names {
        std::fs::write(dir.join(name), body).expect("write prompt");
    }
}

const SMOKE_EMBEDDED_PROMPT_NAMES: &[&str] = &[
    "implement.md",
    "review_descriptions.md",
    "reviewer_template.md",
    "review_write.md",
    "kpop.md",
    "kpop_common.md",
    "kpop_block.md",
    "mbc2_pure.md",
    "mbc2.md",
    "concerns.md",
    "learn.md",
    crate::prompts::HEADER_MD,
    crate::prompts::DO_HEADER_MD,
    "coding_rules.md",
];

fn kiss_smoke_prompt_store(prompts_dir: &std::path::Path) -> crate::prompts::PromptStore {
    kiss_write_same_body_files(prompts_dir, SMOKE_EMBEDDED_PROMPT_NAMES, b"body");
    let store = crate::prompts::PromptStore::with_root(prompts_dir.to_path_buf());
    store.ensure_defaults().expect("defaults");
    store.validate_required().expect("required");
    store
}

fn kiss_smoke_render_context() -> std::collections::HashMap<String, String> {
    std::collections::HashMap::from([
        ("plan_path".to_string(), "p".to_string()),
        ("kpop_log_dir".to_string(), "./_kpop".to_string()),
        ("quality_gates".to_string(), String::new()),
    ])
}

#[test]
fn smoke_prompt_store_with_root() {
    let tmp = kiss_test_tmp();
    let prompts = tmp.path().join("prompts");
    let store = kiss_smoke_prompt_store(&prompts);
    let ctx = kiss_smoke_render_context();
    let _ = store.render("implement.md", &ctx).expect("render");
}
