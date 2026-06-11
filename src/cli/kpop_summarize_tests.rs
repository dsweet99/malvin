#![allow(unsafe_code)]

use std::collections::HashMap;
use std::path::Path;

use crate::artifacts::create_kpop_run_artifacts;
use crate::cli::kpop_summarize::{
    exp_log_paths_markdown, insert_summarize_log_context, is_written_exp_log_path,
    kpop_outer_loop_summarize_params, list_written_exp_logs, outer_loop_summarize_warranted,
    render_kpop_summarize_prompt, run_outer_loop_summarize_if_warranted, run_summarize_coder_prompt,
    KpopOuterLoopSummarizeInputs,
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

fn kpop_inputs<'a>(max_loops: usize, shared: &'a SharedOpts) -> KpopOuterLoopSummarizeInputs<'a> {
    KpopOuterLoopSummarizeInputs {
        max_loops,
        agent_ran: true,
        shared,
    }
}

fn summarize_test_workspace() -> (tempfile::TempDir, crate::artifacts::RunArtifacts, PromptStore, SharedOpts)
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
    let params = kpop_outer_loop_summarize_params(kpop_inputs(2, &shared), &store, &artifacts);
    assert_eq!(params.max_loops, 2);
    assert!(params.agent_ran);
    assert_eq!(params.malvin_command, "malvin kpop");
    assert!(!params.workflow.force);
    assert!(std::ptr::eq(params.store, &raw const store));
    assert!(std::ptr::eq(params.artifacts, &raw const artifacts));
}

#[test]
fn outer_loop_summarize_warranted_only_when_max_loops_gt_one() {
    assert!(!outer_loop_summarize_warranted(0));
    assert!(!outer_loop_summarize_warranted(1));
    assert!(outer_loop_summarize_warranted(2));
}

#[test]
fn render_kpop_summarize_prompt_includes_activity_heading() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join(".malvin")).expect("mkdir");
    let artifacts = create_kpop_run_artifacts("kpop", Some(tmp.path())).expect("artifacts");
    let store = PromptStore::default_store();
    store.ensure_defaults().expect("defaults");
    let prompt = render_kpop_summarize_prompt(&store, &artifacts, "malvin kpop", 2).expect("render");
    assert!(prompt.contains("Summarize the activity"));
    assert!(prompt.contains("Executive summary"));
    assert!(!prompt.contains("{{"));
    assert!(
        prompt.contains(".malvin/logs"),
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

#[test]
fn run_outer_loop_summarize_if_warranted_skips_when_agent_did_not_run() {
    let (_tmp, artifacts, store, shared) = summarize_test_workspace();
    let mut params = kpop_outer_loop_summarize_params(kpop_inputs(2, &shared), &store, &artifacts);
    params.agent_ran = false;
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    rt.block_on(async {
        run_outer_loop_summarize_if_warranted(&params)
            .await
            .expect("skip");
    });
    assert!(!artifacts.log_path("summary").exists());
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
fn insert_summarize_log_context_populates_expected_keys() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join(".malvin")).expect("mkdir");
    let artifacts = create_kpop_run_artifacts("kpop", Some(tmp.path())).expect("artifacts");
    let mut ctx = HashMap::new();
    insert_summarize_log_context(&mut ctx, &artifacts, 2);
    assert!(ctx.contains_key("kpop_log"));
    assert!(ctx.contains_key("stdout_log"));
    assert!(ctx.contains_key("command_log"));
    assert!(ctx.contains_key("exp_log_paths"));
    assert_eq!(ctx.get("outer_loop_count").map(String::as_str), Some("2"));
}

#[test]
fn is_written_exp_log_path_filters_non_matching_names() {
    assert!(is_written_exp_log_path(Path::new("exp_log_run_g1.md")));
    assert!(!is_written_exp_log_path(Path::new("exp_log_run_g1.txt")));
}

#[test]
fn run_outer_loop_summarize_if_warranted_skips_single_loop() {
    let (_tmp, artifacts, store, shared) = summarize_test_workspace();
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    rt.block_on(async {
        run_outer_loop_summarize_if_warranted(&kpop_outer_loop_summarize_params(
            kpop_inputs(1, &shared),
            &store,
            &artifacts,
        ))
        .await
        .expect("skip");
    });
    assert!(!artifacts.log_path("summary").exists());
}

#[test]
fn list_written_exp_logs_collects_kpop_dir_md_files() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let kpop_dir = tmp.path().join("_kpop");
    std::fs::create_dir_all(&kpop_dir).expect("mkdir");
    std::fs::write(kpop_dir.join("exp_log_a.md"), "").expect("write");
    std::fs::write(kpop_dir.join("notes.txt"), "").expect("write");
    let paths = list_written_exp_logs(tmp.path());
    assert_eq!(paths.len(), 1);
    assert!(paths[0].ends_with("exp_log_a.md"));
}
