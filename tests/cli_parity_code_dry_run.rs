//! `malvin code --dry-run` check_plan-only workflow.

mod common;

#[cfg(unix)]
use common::{
    CodeRunOpts, acp_mock_code_dry_run_check_plan_lgtm_js,
    acp_mock_code_dry_run_check_plan_rejects_js, combined_cli_output,
    run_code_with_mock_js_trust_plan, test_home_workspace,
};

#[cfg_attr(unix, test)]
fn dry_run_runs_check_plan_and_stops_before_implement_on_lgtm() {
    let out = run_code_with_mock_js_trust_plan(
        &acp_mock_code_dry_run_check_plan_lgtm_js(),
        &["--dry-run", "--no-tee"],
        &CodeRunOpts {
            no_tee: true,
            trust_plan: false,
        },
    );
    let combined = combined_cli_output(&out);
    assert!(
        out.status.success(),
        "dry-run should succeed when check_plan writes LGTM: {combined:?}"
    );
    assert!(
        combined.contains("CheckPlan"),
        "expected check_plan phase: {combined:?}"
    );
    assert!(
        !combined.contains("implement_phase_ran"),
        "implement must not run in dry-run: {combined:?}"
    );
    assert!(
        !combined.contains("review_phase_ran"),
        "review must not run in dry-run: {combined:?}"
    );
    assert!(
        !combined.contains("summary_phase_ran"),
        "summary must not run in dry-run: {combined:?}"
    );
    assert!(
        combined.contains("DONE"),
        "expected normal completion banner: {combined:?}"
    );
    assert!(
        combined.contains("Plan check passed"),
        "dry-run must report LGTM on stdout when check_plan accepts the plan: {combined:?}"
    );
}

#[cfg_attr(unix, test)]
fn dry_run_fails_when_check_plan_rejects_plan() {
    let out = run_code_with_mock_js_trust_plan(
        &acp_mock_code_dry_run_check_plan_rejects_js(),
        &["--dry-run", "--no-tee"],
        &CodeRunOpts {
            no_tee: true,
            trust_plan: false,
        },
    );
    let combined = combined_cli_output(&out);
    assert!(
        !out.status.success(),
        "dry-run should fail when check_plan rejects the plan: {combined:?}"
    );
    assert!(
        combined.contains("Plan check failed") || combined.contains("check_plan did not pass"),
        "expected check_plan rejection message: {combined:?}"
    );
    assert!(
        !combined.contains("implement_phase_ran"),
        "implement must not run after check_plan rejection in dry-run: {combined:?}"
    );
}

#[cfg_attr(unix, test)]
fn dry_run_conflicts_with_trust_the_plan() {
    let (_root, home, workspace) = test_home_workspace();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_malvin"))
        .current_dir(&workspace)
        .env("HOME", &home)
        .args(["code", "--dry-run", "--trust-the-plan", "ship it"])
        .output()
        .expect("spawn malvin code");
    assert!(
        !out.status.success(),
        "expected clap conflict for --dry-run with --trust-the-plan: {out:?}"
    );
    let combined = combined_cli_output(&out);
    assert!(
        combined.contains("cannot be used with"),
        "expected clap mutual-exclusion error: {combined:?}"
    );
}
