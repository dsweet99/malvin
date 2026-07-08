use crate::cli::WorkflowCliOptions;
use crate::prompts::PromptStore;

use super::*;

fn priors_kpop_request_in_workspace(tmp: &std::path::Path, out: &std::path::Path) -> String {
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("priors", Some(tmp)).expect("artifacts");
    let store = PromptStore::default_store();
    store.ensure_defaults().expect("defaults");
    let user_req = artifacts.run_dir.join("user_request.md");
    std::fs::write(&user_req, "sample request\n").expect("write user request");
    priors_kpop_request(&store, &artifacts, out, &user_req).expect("request")
}

#[test]
fn default_constraints_prompt_embeds_priors() {
    assert!(crate::prompts::default_file("priors_constraints.md").is_some());
}

#[test]
fn default_prompts_list_includes_priors_constraints() {
    assert!(crate::prompts::DEFAULT_PROMPTS.contains(&"priors_constraints.md"));
}

#[test]
fn prepare_priors_kpop_prompt_store_loads_program_and_constraints() {
    let workflow = WorkflowCliOptions { force: false };
    let store = prepare_priors_kpop_prompt_store(workflow).expect("store");
    assert!(store.validate_exists("kpop_program_creative.md").is_ok());
    assert!(store.validate_exists("priors_constraints.md").is_ok());
}

#[test]
fn priors_kpop_request_has_no_unresolved_braces() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("priors.md");
    let text = priors_kpop_request_in_workspace(tmp.path(), &out);
    assert!(
        !text.contains("{{"),
        "priors kpop request must expand all placeholders: {text:?}"
    );
}

#[test]
fn priors_kpop_request_includes_out_priors_path() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("reports/priors.md");
    let text = priors_kpop_request_in_workspace(tmp.path(), &out);
    assert!(
        text.contains("reports/priors.md") || text.contains("./reports/priors.md"),
        "expected out_priors_path in request: {text:?}"
    );
}

#[test]
fn priors_kpop_request_includes_user_request_path() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("priors.md");
    let text = priors_kpop_request_in_workspace(tmp.path(), &out);
    assert!(
        text.contains("user_request.md"),
        "expected user_request_path in request: {text:?}"
    );
}

#[test]
fn priors_kpop_request_includes_kpop_program_wrapper() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("priors.md");
    let text = priors_kpop_request_in_workspace(tmp.path(), &out);
    assert!(
        text.contains("Satisfy all constraints"),
        "expected kpop_program wrapper: {text:?}"
    );
}

#[test]
fn priors_kpop_request_omits_workspace_quality_gates() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("priors.md");
    let text = priors_kpop_request_in_workspace(tmp.path(), &out);
    assert!(
        !text.contains("Quality Gates:"),
        "priors request must omit workspace quality gates: {text:?}"
    );
}
