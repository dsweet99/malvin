#![allow(unsafe_code)]

use std::collections::HashMap;
use std::path::Path;

use crate::artifacts::create_kpop_run_artifacts;
use crate::cli::kpop_summarize::{
    exp_log_paths_markdown, insert_summarize_log_context, is_written_exp_log_path,
    list_written_exp_logs, outer_loop_summarize_warranted,
    prefer_gate_outcome_over_summarize, render_kpop_summarize_prompt,
    run_outer_loop_summarize_if_warranted, run_summarize_coder_prompt, OuterLoopSummarizeParams,
};
use crate::cli::{SharedOpts, WorkflowCliOptions};
use crate::config::{DEFAULT_CLI_MODEL, DEFAULT_MAX_ACP_RETRIES};
use crate::prompts::PromptStore;

fn summarize_shared_opts(max_acp_retries: u32) -> SharedOpts {
    SharedOpts {
        model: DEFAULT_CLI_MODEL.into(),
        no_force: true,
        no_tee: true,
        no_markdown: true,
        verbose: false,
        max_acp_retries,
        doc: false,
    }
}

fn summarize_params<'a>(
    max_loops: usize,
    shared: &'a SharedOpts,
    store: &'a PromptStore,
    artifacts: &'a crate::artifacts::RunArtifacts,
) -> OuterLoopSummarizeParams<'a> {
    OuterLoopSummarizeParams {
        max_loops,
        agent_ran: true,
        shared,
        workflow: WorkflowCliOptions { force: false },
        store,
        artifacts,
        malvin_command: "malvin kpop",
    }
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
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join(".malvin")).expect("mkdir");
    let artifacts = create_kpop_run_artifacts("kpop", Some(tmp.path())).expect("artifacts");
    let store = PromptStore::default_store();
    store.ensure_defaults().expect("defaults");
    let shared = summarize_shared_opts(DEFAULT_MAX_ACP_RETRIES);
    let mut params = summarize_params(2, &shared, &store, &artifacts);
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

fn write_mock_summarize_agent(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

    let handler = r"    const promptText = (((msg.params || {}).prompt || [])[0] || {}).text || '';
    if (promptText.includes('Summarize the activity')) {
      const fs = require('fs');
      const path = require('path');
      fs.appendFileSync(path.join(process.cwd(), 'summary_probe.log'), promptText);
    }
    console.log(JSON.stringify({ jsonrpc: '2.0', method: 'session/update', params: { update: { sessionUpdate: 'agent_message_chunk', content: { type: 'text', text: 'summary\n' } } } }));";
    std::fs::write(path, format!("#!/usr/bin/env node\n{}\n", crate::acp_mock_js("", handler)))
        .expect("write mock");
    let mut perms = std::fs::metadata(path).expect("meta").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).expect("chmod");
}

fn with_summarize_mock_agent<F>(f: F)
where
    F: FnOnce(&std::path::Path, &PromptStore, &crate::artifacts::RunArtifacts),
{
    crate::test_utils::with_isolated_home(|workspace| {
        std::fs::create_dir_all(workspace.join(".malvin")).expect("mkdir");
        let artifacts = create_kpop_run_artifacts("kpop", Some(workspace)).expect("artifacts");
        let store = PromptStore::default_store();
        store.ensure_defaults().expect("defaults");
        let mock = workspace.join("mock-summarize-agent");
        write_mock_summarize_agent(&mock);
        unsafe {
            std::env::set_var("MALVIN_AGENT_ACP_BIN", &mock);
            std::env::set_var("CURSOR_AGENT_API_KEY", "test-key");
        }
        f(workspace, &store, &artifacts);
    });
}

#[test]
fn run_outer_loop_summarize_if_warranted_runs_mock_summary_agent() {
    with_summarize_mock_agent(|workspace, store, artifacts| {
        let shared = summarize_shared_opts(DEFAULT_MAX_ACP_RETRIES);
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        rt.block_on(async {
            run_outer_loop_summarize_if_warranted(&summarize_params(2, &shared, store, artifacts))
                .await
                .expect("summarize");
        });
        let probe = workspace.join("summary_probe.log");
        assert!(probe.is_file(), "mock summarize agent should run");
        let text = std::fs::read_to_string(probe).expect("read probe");
        assert!(text.contains("Summarize the activity"));
        assert!(text.contains("Executive summary"));
        assert!(artifacts.log_path("summary").is_file());
    });
}

#[test]
fn run_summarize_coder_prompt_errors_without_open_session() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = create_kpop_run_artifacts("kpop", Some(tmp.path())).expect("artifacts");
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    rt.block_on(async {
        let mut client = crate::acp::AgentClient::with_max_acp_retries(
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
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join(".malvin")).expect("mkdir");
    let artifacts = create_kpop_run_artifacts("kpop", Some(tmp.path())).expect("artifacts");
    let store = PromptStore::default_store();
    store.ensure_defaults().expect("defaults");
    let shared = summarize_shared_opts(DEFAULT_MAX_ACP_RETRIES);
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    rt.block_on(async {
        run_outer_loop_summarize_if_warranted(&summarize_params(1, &shared, &store, &artifacts))
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

