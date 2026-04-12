//! Behavioral constraints for the CLI.
//!
//! ## Gitignore parity
//!
//! `git check-ignore` guards for the repo root `.gitignore` and the embedded `malvin init` template.
//! Patterns must not use a `./` prefix: git normalizes pathspecs without `./`, so those entries never
//! matched.
//!
//! ## Grounding vs post-run metrics hint
//!
//! Contract checks between `grounding.md` and `src/post_run_hint/report.rs` after git-based metering removal.

use malvin::post_run_hint::POST_RUN_METRICS_NOT_MEASURED_MESSAGE;
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
fn not_measured_message_does_not_blame_git_after_git_metering_removed() {
    assert!(
        !POST_RUN_METRICS_NOT_MEASURED_MESSAGE.contains("git"),
        "git tree metering was removed (see plan); stderr hint must not reference git ({POST_RUN_METRICS_NOT_MEASURED_MESSAGE:?})"
    );
}

#[test]
fn grounding_post_run_hint_stderr_matches_report_implementation() {
    let grounding = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/grounding.md"));
    let report_rs = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/post_run_hint/report.rs"
    ));
    let grounding_ties_post_run_hint_to_stderr = grounding.lines().any(|line| {
        let lower = line.to_lowercase();
        line.contains("stderr") && lower.contains("tracked edit metrics")
    });
    let report_emits_hint_on_stderr = report_rs.lines().any(|line| {
        let t = line.trim_start();
        if t.starts_with("//") || t.starts_with("//!") {
            return false;
        }
        line.contains("eprintln!")
            && line.contains("POST_RUN_METRICS_NOT_MEASURED_MESSAGE")
    });
    assert_eq!(
        grounding_ties_post_run_hint_to_stderr,
        report_emits_hint_on_stderr,
        "grounding.md must document tracked edit metrics on stderr iff report.rs uses eprintln! for that hint"
    );
}

#[test]
fn grounding_run_timing_stderr_contract_matches_run_timing_module() {
    let grounding = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/grounding.md"));
    let run_timing_rs = concat!(
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/run_timing/mod.rs"
        )),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/run_timing/report.rs"
        )),
    );
    let grounding_promises = grounding.contains("run_timing.json") && grounding.contains("stderr");
    let implementation_eprints = run_timing_rs.contains("eprintln!")
        && run_timing_rs.contains("RUN_TIMING_SUMMARY_PREFIX");
    assert_eq!(
        grounding_promises,
        implementation_eprints,
        "grounding.md and src/run_timing/mod.rs + report.rs must stay aligned on run_timing.json + stderr summary"
    );
}

#[test]
fn grounding_kpop_finishes_run_timing_before_post_run_hint() {
    let grounding = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/grounding.md"));
    assert!(
        grounding.contains("KPOP") && grounding.to_lowercase().contains("same ordering"),
        "grounding.md must document KPOP stderr ordering relative to post-run metrics hint"
    );
    let kpop_flow = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/cli/kpop_flow.rs"
    ));
    // Match call sites only (imports appear earlier and would invert ordering).
    let i_finalize = kpop_flow.find("run_timing::finalize_and_emit_run_timing");
    let i_hint = kpop_flow.find("finish_post_run_hint_then_return(&ctx.artifacts.run_dir");
    assert!(
        i_finalize.is_some() && i_hint.is_some(),
        "malvin kpop must call finalize_and_emit_run_timing and finish_post_run_hint_then_return"
    );
    assert!(
        i_finalize.unwrap() < i_hint.unwrap(),
        "finalize_and_emit_run_timing must run before finish_post_run_hint_then_return (grounding.md)"
    );
}
