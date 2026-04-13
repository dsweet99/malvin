//! Behavioral constraints for the CLI.
//!
//! ## Gitignore parity
//!
//! `git check-ignore` guards for the repo root `.gitignore` and the embedded `malvin init` template.
//! Patterns must not use a `./` prefix: git normalizes pathspecs without `./`, so those entries never
//! matched.
//!
//! ## Grounding vs run timing
//!
//! Contract checks between `grounding.md` and the run-timing implementation.

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
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/acp/ops_body.inc")),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/acp/client_impl.inc"
        )),
    )
}

#[test]
fn reviewer_pair_ops_preserves_review_sync_lgtm_before_kpop_order() {
    let ops = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/acp/ops_body.inc"));
    let review = ops
        .find("s.prompt(&review_full, pair.review_log, pair.review_who)")
        .expect("expected review session/prompt in run_reviewer_pair_once");
    let sync = ops
        .find("sync_review_then_is_lgtm(pair.workspace_review_path, pair.artifact_review_path)")
        .expect("expected sync_review_then_is_lgtm after review prompt");
    let kpop = ops
        .find("s.prompt(pair.kpop_body, pair.kpop_log, pair.kpop_who)")
        .expect("expected kpop session/prompt after LGTM branch");
    assert!(
        review < sync && sync < kpop,
        "review prompt must precede workspace→artifact sync/LGTM check, which must precede kpop prompt"
    );
}

#[test]
fn default_cli_model_is_composer_2() {
    let shared = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/cli/shared_opts.rs"
    ));
    assert!(
        shared.contains("const DEFAULT_CLI_MODEL")
            && shared.contains("\"composer-2\"")
            && shared.contains("default_value = DEFAULT_CLI_MODEL"),
        "default `--model` must remain composer-2 via DEFAULT_CLI_MODEL unless intentionally changed"
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
        "/src/acp/client_impl.inc"
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
fn llm_style_docs_do_not_reference_removed_post_run_hint_module() {
    // Regression: `.llm_style/` used to describe `src/post_run_hint/` after that code was removed;
    // keep guides aligned with root `grounding.md` (see `grounding_no_longer_promises_post_run_metrics_hint`).
    for (path, src) in [
        (
            ".llm_style/malvin_tooling.md",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/.llm_style/malvin_tooling.md"
            )),
        ),
        (
            ".llm_style/style.md",
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/.llm_style/style.md")),
        ),
    ] {
        assert!(
            !src.contains("src/post_run_hint") && !src.contains("post_run_hint/"),
            "{path} must not reference removed post_run_hint paths; align with grounding.md",
        );
    }
}

#[test]
fn malvin_tooling_documents_run_artifacts_module_dir_not_flat_file() {
    let tooling = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/.llm_style/malvin_tooling.md"
    ));
    assert!(
        !tooling.contains("src/artifacts.rs"),
        "malvin_tooling.md must not point at removed flat `src/artifacts.rs`; run artifacts live under `src/artifacts/`",
    );
    assert!(
        tooling.contains("src/artifacts/mod.rs") && tooling.contains("src/artifacts/"),
        "malvin_tooling.md must document `RunArtifacts` / review paths via `src/artifacts/` (e.g. mod.rs)",
    );
}

#[test]
fn malvin_do_raw_skips_repo_style_prepend_contract() {
    let do_flow = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/cli/do_flow.rs"));
    let client_impl = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/acp/client_impl.inc"
    ));
    assert!(
        do_flow.contains("skip_repo_style") && do_flow.contains("do_args.raw"),
        "`malvin do --raw` must pass skip_repo_style from do_args.raw into run_coder_prompt (no .style/main.md prepend)"
    );
    assert!(
        client_impl.contains("skip_repo_style")
            && client_impl.contains("compose_coder_prompt_for_session"),
        "AgentClient::run_coder_prompt must honor skip_repo_style"
    );
}

#[test]
fn grounding_run_timing_stdout_contract_matches_run_timing_module() {
    let grounding = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/grounding.md"));
    let report_rs = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/run_timing/report.rs"
    ));
    let grounding_promises =
        grounding.contains("run_timing.json") && grounding.contains("**stdout** summary line");
    // Require the stdout + JSON contract in `report.rs` (not module docs / `mod.rs` alone).
    let implementation_delivers = report_rs.contains("write_json_and_print_summary")
        && report_rs.contains("print_stdout_line")
        && report_rs.contains("RUN_TIMING_SUMMARY_PREFIX")
        && report_rs.contains("RUN_TIMING_JSON_FILE");
    assert_eq!(
        grounding_promises, implementation_delivers,
        "grounding.md and src/run_timing/report.rs must stay aligned on run_timing.json + stdout summary"
    );
}

#[test]
fn grounding_no_longer_promises_post_run_metrics_hint() {
    let grounding = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/grounding.md"));
    assert!(
        !grounding.to_lowercase().contains("tracked edit metrics"),
        "grounding.md should not mention the removed post-run metrics hint"
    );
    assert!(
        !agent_sources_for_snapshot().contains("post_run_hint"),
        "ACP/workflow sources should not reference the removed post-run metrics hint"
    );
}

#[test]
fn shared_opts_and_run_timing_sources_must_not_revive_stderr_post_run_metrics_copy() {
    let shared = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/cli/shared_opts.rs"
    ));
    let run_timing = concat!(
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/run_timing/mod.rs"
        )),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/run_timing/report.rs"
        )),
    );
    let kpop_flow = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/cli/kpop_flow.rs"));
    assert!(
        !shared.contains("metrics hint") && !shared.contains("tracked-edit"),
        "`--no-tee` help must not promise removed stderr tracked-edit metrics; align with grounding.md"
    );
    assert!(
        !run_timing.contains("stderr post-run hint") && !run_timing.contains("stderr post-run"),
        "run_timing sources should not describe a stderr post-run metrics line; run timing is stdout + JSON per grounding.md"
    );
    assert!(
        !kpop_flow.contains("post-run hint"),
        "kpop flow comments must not describe a removed stderr post-run metrics step"
    );
}
