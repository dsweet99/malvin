use crate::artifacts::create_kpop_run_artifacts;
use crate::cli::kpop_summarize::{
    insert_summarize_log_context, kpop_flows_ran, list_written_exp_logs,
    maybe_run_gate_inline_summarize, should_inline_outer_loop_summarize_on_gate_iteration,
    GateInlineSummarizeCtx,
};
use crate::cli::WorkflowCliOptions;
use crate::prompt_stratification::WorkflowRenderContext;
use super::kpop_summarize_tests::{summarize_test_workspace, write_exp_logs};

#[test]
fn gate_iteration_inline_summarize_predicate() {
    assert!(!should_inline_outer_loop_summarize_on_gate_iteration(1, 3));
    assert!(!should_inline_outer_loop_summarize_on_gate_iteration(2, 3));
    assert!(should_inline_outer_loop_summarize_on_gate_iteration(3, 3));
}

#[test]
fn insert_summarize_log_context_populates_expected_keys() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join(".malvin")).expect("mkdir");
    let artifacts = create_kpop_run_artifacts("kpop", Some(tmp.path())).expect("artifacts");
    let mut ctx = WorkflowRenderContext::default();
    insert_summarize_log_context(&mut ctx, &artifacts, 2);
    assert!(ctx.contains_key("kpop_log"));
    assert!(ctx.contains_key("stdout_log"));
    assert!(ctx.contains_key("command_log"));
    assert!(ctx.contains_key("exp_log_paths"));
    assert_eq!(ctx.get("outer_loop_count").map(String::as_str), Some("2"));
}

#[test]
fn kpop_flows_ran_counts_written_exp_logs() {
    let (_tmp, artifacts, _store, _shared) = summarize_test_workspace();
    assert_eq!(kpop_flows_ran(&artifacts), 0);
    write_exp_logs(&artifacts, 1);
    assert_eq!(kpop_flows_ran(&artifacts), 1);
    write_exp_logs(&artifacts, 2);
    assert_eq!(kpop_flows_ran(&artifacts), 2);
}

#[test]
fn list_written_exp_logs_collects_kpop_dir_md_files() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let kpop_dir = tmp.path().join("_kpop");
    std::fs::create_dir_all(&kpop_dir).expect("mkdir");
    std::fs::write(kpop_dir.join("exp_log_a.md"), "step\n").expect("write");
    std::fs::write(kpop_dir.join("notes.txt"), "").expect("write");
    let paths = list_written_exp_logs(tmp.path());
    assert_eq!(paths.len(), 1);
    assert!(paths[0].ends_with("exp_log_a.md"));
}

#[cfg(unix)]
async fn run_gate_inline_summarize_first_iteration(
    store: &crate::prompts::PromptStore,
    artifacts: &crate::artifacts::RunArtifacts,
) -> Result<(), String> {
    use crate::config::DEFAULT_MAX_ACP_RETRIES;

    let shared = super::kpop_summarize_tests::summarize_shared_opts(DEFAULT_MAX_ACP_RETRIES);
    let mut client = crate::agent_backend::build_agent_backend(
        &shared,
        WorkflowCliOptions { force: false, no_kpop: false },
        false,
        "kpop",
    )
    .map_err(|e| e.to_string())?;
    client.ensure_authenticated().map_err(|e| e.to_string())?;
    client
        .begin_coder_session(&artifacts.work_dir)
        .await
        .map_err(|e| e.to_string())?;
    maybe_run_gate_inline_summarize(GateInlineSummarizeCtx {
        client: &mut client,
        store,
        artifacts,
        model: shared.model.as_str(),
        git: false,
        iteration: 1,
        total_iterations: 3,
    })
    .await?;
    client.end_coder_session().await.map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn maybe_run_gate_inline_summarize_skips_when_not_last_iteration() {
    super::kpop_summarize_mock_tests::with_summarize_mock_agent(|_workspace, store, artifacts| {
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        rt.block_on(async {
            run_gate_inline_summarize_first_iteration(store, artifacts)
                .await
                .expect("skip");
        });
        assert!(!artifacts.log_path("summary").exists());
    });
}
