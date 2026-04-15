use std::path::Path;
use std::process::Command;

const ROOT_GITIGNORE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/.gitignore"));
const INIT_TEMPLATE_GITIGNORE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/default_repo/gitignore"
));

fn check_ignored(repo: &Path, rel_path: &str) -> bool {
    Command::new("git")
        .current_dir(repo)
        .args(["check-ignore", "-q", rel_path])
        .status()
        .unwrap_or_else(|e| panic!("git check-ignore spawn failed: {e}"))
        .success()
}

#[test]
fn max_loops_zero_must_not_be_clamped_to_one() {
    let max_loops = 0_usize;
    let iterations = (1..=max_loops).count();
    assert_eq!(
        iterations, 0,
        "range(1..=max_loops) yields zero iterations when max_loops is 0"
    );
    let main_rs = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/main.rs"));
    assert!(
        !main_rs.contains("args.max_loops.max(1)"),
        "main.rs must not clamp max_loops with .max(1); that breaks the intended zero-iteration behavior"
    );
    let cli_rs = concat!(
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/cli/mod.rs")),
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/cli/args.rs")),
    );
    assert!(
        !cli_rs.contains("max_loops.max(1)"),
        "src/cli must not clamp max_loops with .max(1); that breaks the intended zero-iteration behavior"
    );
}

const fn agent_sources_for_snapshot() -> &'static str {
    concat!(
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/acp/ops_body.rs")),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/acp/client_impl.rs"
        )),
    )
}

#[test]
fn reviewer_pair_ops_calls_review_prompt() {
    let ops = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/acp/ops_body.rs"));
    ops.find("pair.review_log, pair.review_who, None")
        .expect("expected review session/prompt in run_reviewer_pair_once");
}

#[test]
fn default_cli_model_is_composer_2_fast() {
    let shared = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/cli/shared_opts.rs"
    ));
    assert!(
        shared.contains("const DEFAULT_CLI_MODEL")
            && shared.contains("\"composer-2-fast\"")
            && shared.contains("default_value = DEFAULT_CLI_MODEL"),
        "default `--model` must remain composer-2-fast via DEFAULT_CLI_MODEL unless intentionally changed"
    );
    let models = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/cli/models_cmd.rs"
    ));
    assert!(
        models.contains("DEFAULT_CLI_MODEL")
            && models.contains("Default model in malvin: {DEFAULT_CLI_MODEL}"),
        "`malvin models` footer must use DEFAULT_CLI_MODEL (same string as SharedOpts default)"
    );
}

#[test]
fn agent_client_must_apply_force_when_invoking_acp() {
    let snapshot = agent_sources_for_snapshot();
    assert!(
        !snapshot.contains("let _ = self.force;"),
        "force is stored on AgentClient but discarded before spawn; --force should be passed to cursor-agent when force is true"
    );
    assert!(
        snapshot.contains("force: client.io.force"),
        "spawn must pass `agent --force` via client.io.force"
    );
}

#[test]
fn agent_client_must_apply_model_when_invoking_acp_or_drop_cli_option() {
    let snapshot = agent_sources_for_snapshot();
    assert!(
        !snapshot.contains("let _ = self.model;"),
        "model is accepted on the CLI but discarded before spawn; wire through ACP or document-only at the type level"
    );
    assert!(
        snapshot.contains("model_opt"),
        "spawn must pass model into AcpSpawnArgs"
    );
}

#[test]
fn agent_client_must_apply_tee_mode_when_invoking_acp() {
    let snapshot = agent_sources_for_snapshot();
    assert!(
        snapshot.contains("tee_trace_stdout: !client.io.no_tee"),
        "spawn must pass CLI tee mode into AcpSpawnArgs so trace lines can stream to stdout when tee is on"
    );
}

#[test]
fn upgrade_plan_message_must_not_be_eprint_twice() {
    let client_impl = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/acp/client_impl.rs"
    ));
    let cli_mod = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/cli/mod.rs"));
    let client_eprints_on_upgrade = client_impl.contains("agent_string_is_upgrade_plan")
        && client_impl.contains("eprintln!(\"{last_error}\")");
    let cli_eprints_run_error = cli_mod.contains("eprintln!(\"{e}\")");
    assert!(
        !(client_eprints_on_upgrade && cli_eprints_run_error),
        "upgrade-plan failures eprintln in client_impl and the CLI entrypoint prints AgentError again; stderr duplicates the same message"
    );
}

#[test]
fn root_gitignore_ignores_malvin_logs_and_target() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(
        check_ignored(root, "_malvin/dummy_stamp/plan.md"),
        "expected _malvin/ run dirs to be ignored"
    );
    assert!(
        check_ignored(root, "log"),
        "expected root log file to be ignored"
    );
    assert!(
        check_ignored(root, "log_2"),
        "expected root log_2 to be ignored"
    );
    assert!(
        check_ignored(root, "target/debug/malvin"),
        "expected Rust target/ tree to be ignored"
    );
    assert!(
        !check_ignored(root, "README.md"),
        "expected README.md not to be ignored"
    );
}

#[test]
fn init_template_gitignore_is_consistent_with_git_check_ignore() {
    const TEMPLATE: &str = INIT_TEMPLATE_GITIGNORE;
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join(".gitignore"), TEMPLATE).unwrap();
    let st = Command::new("git")
        .args(["init"])
        .current_dir(tmp.path())
        .status()
        .expect("git init");
    assert!(st.success(), "git init failed");
    assert!(
        check_ignored(tmp.path(), "_malvin/x/plan.md"),
        "template should ignore _malvin/ runs"
    );
    assert!(
        check_ignored(tmp.path(), "log"),
        "template should ignore root log"
    );
    assert!(
        check_ignored(tmp.path(), "log_2"),
        "template should ignore root log_2"
    );
    assert!(
        check_ignored(tmp.path(), "target/release/foo"),
        "template should ignore Rust target/"
    );
    assert!(
        !check_ignored(tmp.path(), "src/lib.rs"),
        "template should not ignore normal sources"
    );
    assert!(
        check_ignored(tmp.path(), "pkg/__pycache__/x.py"),
        "template should ignore sources under nested __pycache__ dirs (not only *.pyc)"
    );
    assert!(
        check_ignored(tmp.path(), "lib/foo.pyc"),
        "template should ignore .pyc via **/*.py[cod]"
    );
}

#[test]
fn init_template_gitignore_matches_root_python_ignore_patterns() {
    for line in ["**/__pycache__/", "**/*.py[cod]"] {
        assert!(
            ROOT_GITIGNORE.lines().any(|l| l.trim() == line),
            "repo root .gitignore must list {line:?}"
        );
        assert!(
            INIT_TEMPLATE_GITIGNORE.lines().any(|l| l.trim() == line),
            "malvin init template .gitignore must list {line:?} so new repos match Malvin's own ignores"
        );
    }
}

#[test]
fn artifacts_grounding_backup_module_is_declared_and_source_tracked() {
    let mod_rs = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/artifacts/mod.rs"));
    let backup_rs = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/artifacts/grounding_backup.rs"
    ));
    assert!(
        mod_rs.contains("mod grounding_backup")
            && mod_rs.contains("pub use grounding_backup::")
            && mod_rs.contains("backup_workspace_grounding_if_present"),
        "src/artifacts/mod.rs must declare `mod grounding_backup` and re-export backup/restore"
    );
    assert!(
        backup_rs.contains("pub fn backup_workspace_grounding_if_present")
            && backup_rs.contains("pub fn restore_workspace_grounding")
            && backup_rs.contains("# Errors"),
        "src/artifacts/grounding_backup.rs must ship with backup/restore APIs and documented errors (commit beside mod.rs)",
    );
}

#[test]
fn malvin_do_default_skips_repo_style_prepend_contract() {
    let do_flow = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/cli/do_flow.rs"));
    let client_impl = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/acp/client_impl.rs"
    ));
    assert!(
        do_flow.contains("skip_repo_style") && do_flow.contains("do_args.cooked"),
        "`malvin do` default raw must pass skip_repo_style from !do_args.cooked into run_coder_prompt (no injected repo style prepend)"
    );
    assert!(
        client_impl.contains("skip_repo_style")
            && client_impl.contains("coder_prompt_body_with_optional_repo_style"),
        "AgentClient::run_coder_prompt must honor skip_repo_style"
    );
}

#[test]
fn kpop_p_creative_help_text_matches_creative_min_interaction_contract() {
    let args_rs = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/cli/args.rs"));
    assert!(
        !args_rs.contains("first 3 prompts"),
        "`malvin kpop --p-creative` help must not claim a stale 'first 3 prompts' deferral; align with src/kpop_acp_prompt.rs (CREATIVE_MIN_INTERACTION)"
    );
}

#[test]
fn cargo_package_description_must_not_embed_acp_trace_or_log_artifacts() {
    let cargo_toml = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"));
    let Some(desc_line) = cargo_toml.lines().find(|l| {
        let t = l.trim_start();
        t.starts_with("description = ")
    }) else {
        panic!("Cargo.toml must declare [package] description = \"...\"");
    };
    assert!(
        !desc_line.contains(":[>"),
        "package description must be human-facing crate metadata, not a pasted ACP tee / log line (found `:[>` in {desc_line:?})"
    );
}

#[test]
fn implement_prompt_validate_plan_claim_must_match_workflow_and_grounding() {
    let implement = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/default_prompts/implement.md"
    ));
    let orchestrator = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/orchestrator/mod.rs"
    ));
    let grounding = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/grounding.md"));
    let implement_claims_validate_plan = implement.contains("preceding validate_plan step");
    let workflow_runs_validate_plan = orchestrator.contains("\"validate_plan.md\"");
    let grounding_documents_validate_plan = grounding.contains("validate_plan");
    assert!(
        !implement_claims_validate_plan
            || (workflow_runs_validate_plan && grounding_documents_validate_plan),
        "implement.md must not claim a preceding validate_plan step unless both the workflow driver and grounding.md include that phase"
    );
}

#[test]
fn grounding_documents_conditional_learn_when_workflow_skips_short_runs() {
    let cli_mod = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/cli/mod.rs"));
    let orchestrator = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/orchestrator/mod.rs"
    ));
    let grounding = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/grounding.md"));
    let code_has_conditional_learn = cli_mod.contains("LEARN_MIN_ELAPSED_MS")
        && orchestrator.contains("learn_min_elapsed_ms")
        && orchestrator.contains("should_run_learn()");
    let grounding_mentions_conditional_learn = grounding.contains("5 minutes")
        || grounding.contains("300_000")
        || grounding.contains("Only run the learn phase when")
        || grounding.contains("skip learning")
        || grounding.contains("unless the run is short");
    assert!(
        !code_has_conditional_learn || grounding_mentions_conditional_learn,
        "grounding.md must document the conditional learn gate once the workflow skips learn on short runs"
    );
}
