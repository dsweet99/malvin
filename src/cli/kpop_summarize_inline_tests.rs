#![allow(unsafe_code)]

use crate::artifacts::create_kpop_run_artifacts;
use crate::cli::gate_kpop_workflow::GateLoopBehavior;
use crate::cli::kpop_summarize::{
    code_outer_loop_summarize_params, insert_summarize_log_context, kpop_flows_ran,
    kpop_outer_loop_summarize_params, list_written_exp_logs, maybe_run_gate_inline_summarize,
    maybe_run_inline_summarize_on_kpop_loop, outer_loop_summarize_warranted,
    run_outer_loop_summarize_if_warranted, should_inline_outer_loop_summarize_on_gate_iteration,
    should_inline_outer_loop_summarize_on_kpop_loop, CodeOuterLoopSummarizeInputs,
    GateInlineSummarizeCtx, InlineSummarizeOnKpopLoopCtx,
};
use super::kpop_summarize_tests::{kpop_inputs, summarize_test_workspace, write_exp_logs};

#[test]
fn gate_iteration_inline_summarize_predicate() {
    assert!(!should_inline_outer_loop_summarize_on_gate_iteration(
        1,
        3,
        0,
        GateLoopBehavior::CODE
    ));
    assert!(!should_inline_outer_loop_summarize_on_gate_iteration(
        2,
        3,
        0,
        GateLoopBehavior::CODE
    ));
    assert!(should_inline_outer_loop_summarize_on_gate_iteration(
        2,
        3,
        1,
        GateLoopBehavior::CODE
    ));
    assert!(should_inline_outer_loop_summarize_on_gate_iteration(
        3,
        3,
        0,
        GateLoopBehavior::CODE
    ));
}

#[test]
fn kpop_loop_inline_summarize_predicate() {
    assert!(!should_inline_outer_loop_summarize_on_kpop_loop(1, 2, true));
    assert!(should_inline_outer_loop_summarize_on_kpop_loop(2, 2, true));
    assert!(!should_inline_outer_loop_summarize_on_kpop_loop(2, 5, false));
    assert!(should_inline_outer_loop_summarize_on_kpop_loop(2, 5, true));
}

#[test]
fn run_outer_loop_summarize_if_warranted_is_noop() {
    let (_tmp, artifacts, store, shared) = summarize_test_workspace();
    write_exp_logs(&artifacts, 2);
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    rt.block_on(async {
        run_outer_loop_summarize_if_warranted(&kpop_outer_loop_summarize_params(
            kpop_inputs(&shared),
            &store,
            &artifacts,
        ))
        .await
        .expect("noop");
    });
    assert!(!artifacts.log_path("summary").exists());
}

#[test]
fn outer_loop_summarize_warranted_only_when_kpop_flows_gt_one() {
    assert!(!outer_loop_summarize_warranted(0));
    assert!(!outer_loop_summarize_warranted(1));
    assert!(outer_loop_summarize_warranted(2));
}

#[test]
fn run_outer_loop_summarize_if_warranted_skips_when_agent_did_not_run() {
    let (_tmp, artifacts, store, shared) = summarize_test_workspace();
    write_exp_logs(&artifacts, 2);
    let mut params = kpop_outer_loop_summarize_params(kpop_inputs(&shared), &store, &artifacts);
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
fn run_outer_loop_summarize_if_warranted_skips_single_flow() {
    let (_tmp, artifacts, store, shared) = summarize_test_workspace();
    write_exp_logs(&artifacts, 1);
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    rt.block_on(async {
        run_outer_loop_summarize_if_warranted(&kpop_outer_loop_summarize_params(
            kpop_inputs(&shared),
            &store,
            &artifacts,
        ))
        .await
        .expect("skip");
    });
    assert!(!artifacts.log_path("summary").exists());
}

#[test]
fn insert_summarize_log_context_populates_expected_keys() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join(".malvin")).expect("mkdir");
    let artifacts = create_kpop_run_artifacts("kpop", Some(tmp.path())).expect("artifacts");
    let mut ctx = std::collections::HashMap::new();
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

#[test]
fn code_outer_loop_summarize_params_wires_code_command() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join(".malvin")).expect("mkdir");
    let old = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(tmp.path()).expect("chdir");
    let prepared = super::code_flow::prepare_code_kpop_run(
        crate::cli::WorkflowCliOptions { force: false },
        "ship it",
    )
    .expect("prepared");
    let shared = super::kpop_summarize_tests::summarize_shared_opts(1);
    let params = code_outer_loop_summarize_params(
        CodeOuterLoopSummarizeInputs {
            agent_ran: true,
            shared: &shared,
            workflow: crate::cli::WorkflowCliOptions { force: false },
        },
        &prepared,
    );
    std::env::set_current_dir(old).expect("restore cwd");
    assert_eq!(params.malvin_command, "malvin code");
    assert!(params.agent_ran);
}

#[cfg(unix)]
#[test]
fn maybe_run_inline_summarize_on_kpop_loop_runs_on_last_iteration() {
    super::kpop_summarize_mock_tests::with_summarize_mock_agent(|workspace, store, artifacts| {
        write_exp_logs(artifacts, 2);
        let shared = super::kpop_summarize_tests::summarize_shared_opts(1);
        crate::test_utils::block_on_test_async(async {
            let mut client = crate::agent_backend::build_agent_backend(
                &shared,
                crate::cli::WorkflowCliOptions { force: false },
                false,
                "kpop",
            )
            .map_err(|e| e.to_string())
            .expect("backend");
            client.ensure_authenticated().map_err(|e| e.to_string()).expect("auth");
            maybe_run_inline_summarize_on_kpop_loop(InlineSummarizeOnKpopLoopCtx {
                client: &mut client,
                store,
                artifacts,
                agent_loop: 2,
                max_loops: 2,
                will_exit_after_this_loop: true,
            })
            .await
            .expect("inline summarize");
        });
        assert!(workspace.join("summary_probe.log").is_file());
        assert!(artifacts.log_path("summary").is_file());
    });
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
        crate::cli::WorkflowCliOptions { force: false },
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
        malvin_command: "malvin code",
        iteration: 1,
        total_iterations: 3,
        consecutive_solved_entering: 0,
        behavior: GateLoopBehavior::CODE,
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
