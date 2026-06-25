#![allow(unsafe_code)]

use std::path::Path;

use crate::artifacts::create_kpop_run_artifacts;
use crate::cli::kpop_summarize::{
    exp_log_paths_markdown, is_written_exp_log_path, kpop_outer_loop_summarize_params,
    render_kpop_summarize_prompt, run_summarize_coder_prompt, KpopOuterLoopSummarizeInputs,
};
use crate::cli::workflow_kpop_shared::prefer_gate_outcome_over_summarize;
use crate::cli::SharedOpts;
use crate::config::{DEFAULT_CLI_MODEL, DEFAULT_MAX_ACP_RETRIES};
use crate::prompts::PromptStore;

pub(crate) fn summarize_shared_opts(max_acp_retries: u32) -> SharedOpts {
    SharedOpts {
        model: DEFAULT_CLI_MODEL.into(),
        no_force: true,
        no_tenacious: false,
        no_tee: true,
        no_markdown: true,
        verbose: false,
        max_acp_retries,
        doc: false,
        name: None,
        mini: false,
        mini_max_bash_turns: 32,
    }
}

pub(crate) fn kpop_inputs<'a>(shared: &'a SharedOpts) -> KpopOuterLoopSummarizeInputs<'a> {
    KpopOuterLoopSummarizeInputs {
        agent_ran: true,
        shared,
    }
}

pub(crate) fn summarize_test_workspace() -> (tempfile::TempDir, crate::artifacts::RunArtifacts, PromptStore, SharedOpts)
{
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join(".malvin")).expect("mkdir");
    let artifacts = create_kpop_run_artifacts("kpop", Some(tmp.path())).expect("artifacts");
    let store = PromptStore::default_store();
    store.ensure_defaults().expect("defaults");
    let shared = summarize_shared_opts(DEFAULT_MAX_ACP_RETRIES);
    (tmp, artifacts, store, shared)
}

#[test]
fn kpop_outer_loop_summarize_params_builds_kpop_context() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join(".malvin")).expect("mkdir");
    let artifacts = create_kpop_run_artifacts("kpop", Some(tmp.path())).expect("artifacts");
    let store = PromptStore::default_store();
    store.ensure_defaults().expect("defaults");
    let shared = summarize_shared_opts(DEFAULT_MAX_ACP_RETRIES);
    let params = kpop_outer_loop_summarize_params(kpop_inputs(&shared), &store, &artifacts);
    assert!(params.agent_ran);
    assert_eq!(params.malvin_command, "malvin kpop");
    assert!(!params.workflow.force);
    assert!(std::ptr::eq(params.store, &raw const store));
    assert!(std::ptr::eq(params.artifacts, &raw const artifacts));
}

#[test]
fn render_kpop_summarize_prompt_includes_activity_heading() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join(".malvin")).expect("mkdir");
    let artifacts = create_kpop_run_artifacts("kpop", Some(tmp.path())).expect("artifacts");
    let store = PromptStore::default_store();
    store.ensure_defaults().expect("defaults");
    let prompt = render_kpop_summarize_prompt(&store, &artifacts, "malvin kpop").expect("render");
    assert!(prompt.contains("Summarize the activity"));
    assert!(prompt.contains("Executive summary"));
    assert!(!prompt.contains("{{"));
    assert!(
        prompt.contains(".malvin_home/logs"),
        "summarize header must render logs_dir to home logs bucket"
    );
}

#[test]
fn prefer_gate_outcome_over_summarize_keeps_gate_error() {
    let gate: Result<(), String> = Err("gates exhausted".to_string());
    let summarize = Err("summarize auth failed".to_string());
    let merged = prefer_gate_outcome_over_summarize(gate, summarize).expect_err("gate wins");
    assert_eq!(merged, "gates exhausted");
}

#[test]
fn prefer_gate_outcome_over_summarize_surfaces_summarize_when_gate_ok() {
    let gate: Result<(), String> = Ok(());
    let summarize = Err("summarize failed".to_string());
    let merged = prefer_gate_outcome_over_summarize(gate, summarize).expect_err("summarize");
    assert_eq!(merged, "summarize failed");
}

pub(crate) fn write_exp_logs(artifacts: &crate::artifacts::RunArtifacts, count: usize) {
    let kpop_dir = artifacts.run_dir.join("_kpop");
    std::fs::create_dir_all(&kpop_dir).expect("mkdir");
    for i in 1..=count {
        std::fs::write(kpop_dir.join(format!("exp_log_test_g{i}.md")), "step\n").expect("write");
    }
}

#[test]
fn exp_log_paths_markdown_lists_existing_files() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let kpop_dir = tmp.path().join(".malvin/logs/run/_kpop");
    std::fs::create_dir_all(&kpop_dir).expect("mkdir");
    std::fs::write(kpop_dir.join("exp_log_test_g1.md"), "x").expect("write");
    let artifacts = crate::artifacts::RunArtifacts {
        run_dir: tmp.path().join(".malvin/logs/run"),
        plan_path: tmp.path().join("plan.md"),
        work_dir: tmp.path().to_path_buf(),
    };
    let md = exp_log_paths_markdown(&artifacts);
    assert!(md.contains("exp_log_test_g1.md"));
}

#[test]
fn run_summarize_coder_prompt_errors_without_open_session() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = create_kpop_run_artifacts("kpop", Some(tmp.path())).expect("artifacts");
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    rt.block_on(async {
        let mut client = crate::agent_backend::AgentBackend::Acp(
            crate::acp::AgentClient::with_max_acp_retries(
                "m".into(),
                crate::acp::AgentIoOptions {
                    force: false,
                    no_tee: true,
                    raw_output: true,
                    show_thoughts_on_stdout: false,
                    emit_stdout_markdown: false,
                    log_full_outgoing_prompts: false,
                },
                DEFAULT_MAX_ACP_RETRIES,
            ),
        );
        let err = run_summarize_coder_prompt(&mut client, &artifacts, "Summarize the activity")
            .await
            .expect_err("must fail without begin_coder_session");
        assert!(err.contains("begin_coder_session"));
    });
}

#[test]
fn is_written_exp_log_path_filters_non_matching_names() {
    assert!(is_written_exp_log_path(Path::new("exp_log_run_g1.md")));
    assert!(!is_written_exp_log_path(Path::new("exp_log_run_g1.txt")));
}

