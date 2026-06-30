use std::collections::HashMap;

use super::{
    build_mpc_planner_context, build_mpc_planner_prompt, mpc_enabled, mpc_planner_exp_log_path,
    prepare_mpc_planner_turn, reset_user_brief_before_planner, run_mpc_planner_session,
    user_brief_baseline_path, MpcPlannerParams,
};
use crate::prompt_stratification::WorkflowRenderContext;
use crate::prompts::PromptStore;
use crate::test_utils::with_isolated_home;
use crate::workspace_paths::malvin_config_path;

macro_rules! mpc_prepare_turn_reset_workflow {
    ($work:expr) => {{
        let artifacts =
            crate::artifacts::create_kpop_run_artifacts("plan", Some($work)).expect("artifacts");
        let brief_path = crate::workflow_context::resolve_user_brief_path(
            &artifacts,
            &WorkflowRenderContext::default(),
        );
        std::fs::write(&brief_path, "brief\n").expect("write brief");
        let store = PromptStore::default_store();
        store.ensure_defaults().expect("defaults");
        let (_kpop, shared, workflow) =
            crate::cli::kpop_flow::kpop_flow_run_loop_tests::test_kpop_args(1);
        let base_context =
            crate::cli::workflow_kpop_shared::kpop_workflow_context(&artifacts, "code")
                .expect("context");
        let context = build_mpc_planner_context(&base_context, &artifacts);
        (artifacts, brief_path, store, shared, workflow, context)
    }};
}

fn mpc_test_store() -> (tempfile::TempDir, PromptStore) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path().join("prompts");
    std::fs::create_dir_all(&root).expect("mkdir");
    for (name, body) in [
        ("header.md", "HDR {{ plan_path }}\n"),
        ("kpop_common.md", "COMMON {{ exp_log }}\n"),
        ("mpc_planner.md", "MPC {{ user_request_path }}\n"),
    ] {
        std::fs::write(root.join(name), body).expect("write");
    }
    let store = PromptStore::with_root(root);
    store.ensure_defaults().expect("defaults");
    (tmp, store)
}

#[test]
fn build_mpc_planner_prompt_joins_sections_in_order() {
    let (_tmp, store) = mpc_test_store();
    let ctx = WorkflowRenderContext::from(HashMap::from([
        ("plan_path".to_string(), "./plan.md".to_string()),
        (
            "user_request_path".to_string(),
            "./user_request.md".to_string(),
        ),
        ("exp_log".to_string(), "./_kpop/mpc_planner_log.md".to_string()),
        ("current_state".to_string(), "User: test".to_string()),
    ]));
    let out = build_mpc_planner_prompt(&store, &ctx).expect("prompt");
    let hdr = out.find("HDR").expect("header");
    let common = out.find("COMMON").expect("common");
    let mpc = out.find("MPC").expect("mpc");
    assert!(hdr < common);
    assert!(common < mpc);
    assert!(out.contains("./user_request.md"));
    assert!(!out.contains("{{"));
}

#[test]
fn build_mpc_planner_context_sets_dedicated_exp_log() {
    with_isolated_home(|work| {
        let artifacts =
            crate::artifacts::create_kpop_run_artifacts("code", Some(work)).expect("artifacts");
        let base = WorkflowRenderContext::from(HashMap::from([(
            "user_request_path".to_string(),
            "./user_request.md".to_string(),
        )]));
        let ctx = build_mpc_planner_context(&base, &artifacts);
        let exp_log = ctx.get("exp_log").expect("exp_log");
        assert!(exp_log.contains("mpc_planner_log.md"));
        assert!(ctx.contains_key("current_state"));
    });
}

#[test]
fn mpc_planner_iteration_log_path_suffixes_iteration() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("x", Some(tmp.path())).expect("artifacts");
    let path = super::mpc_planner_iteration_log_path(&artifacts, 2);
    assert!(path.to_string_lossy().contains("mpc_planner_2"));
}

#[test]
fn mpc_planner_exp_log_path_is_under_kpop_dir() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("x", Some(tmp.path())).expect("artifacts");
    let path = mpc_planner_exp_log_path(&artifacts);
    assert!(path.to_string_lossy().contains("_kpop"));
    assert!(path.ends_with("mpc_planner_log.md"));
}

#[test]
fn mpc_enabled_reads_config() {
    with_isolated_home(|work| {
        assert!(mpc_enabled(work));
        let path = malvin_config_path(work);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(&path, "mpc = false\n").expect("write");
        assert!(!mpc_enabled(work));
    });
}

#[test]
fn build_mpc_planner_prompt_errors_on_missing_templates() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let store = PromptStore::with_root(tmp.path().join("empty"));
    assert!(build_mpc_planner_prompt(&store, &WorkflowRenderContext::default()).is_err());
}

#[test]
fn reset_user_brief_before_planner_captures_baseline_on_first_call() {
    with_isolated_home(|work| {
        let artifacts =
            crate::artifacts::create_kpop_run_artifacts("plan", Some(work)).expect("artifacts");
        let brief_path = crate::workflow_context::resolve_user_brief_path(
            &artifacts,
            &WorkflowRenderContext::default(),
        );
        std::fs::write(&brief_path, "original\n").expect("write brief");
        reset_user_brief_before_planner(&artifacts, &WorkflowRenderContext::default())
            .expect("reset");
        let baseline_path = user_brief_baseline_path(&artifacts);
        assert!(baseline_path.is_file());
        assert_eq!(
            std::fs::read_to_string(&baseline_path).expect("read baseline"),
            "original\n"
        );
        assert_eq!(
            std::fs::read_to_string(&brief_path).expect("read brief"),
            "original\n"
        );
    });
}

#[test]
fn reset_user_brief_before_planner_restores_on_second_call() {
    with_isolated_home(|work| {
        let artifacts =
            crate::artifacts::create_kpop_run_artifacts("plan", Some(work)).expect("artifacts");
        let brief_path = crate::workflow_context::resolve_user_brief_path(
            &artifacts,
            &WorkflowRenderContext::default(),
        );
        std::fs::write(&brief_path, "original\n").expect("write brief");
        reset_user_brief_before_planner(&artifacts, &WorkflowRenderContext::default())
            .expect("first reset");
        std::fs::write(&brief_path, "original\nappended\n").expect("mutate brief");
        reset_user_brief_before_planner(&artifacts, &WorkflowRenderContext::default())
            .expect("second reset");
        assert_eq!(
            std::fs::read_to_string(&brief_path).expect("read brief"),
            "original\n"
        );
    });
}

#[test]
fn prepare_mpc_planner_turn_resets_brief_before_second_call() {
    with_isolated_home(|work| {
        let (artifacts, brief_path, store, shared, workflow, context) =
            mpc_prepare_turn_reset_workflow!(work);
        let params = |iteration: Option<usize>| MpcPlannerParams {
            shared: &shared,
            workflow,
            store: &store,
            artifacts: &artifacts,
            context: &context,
            command: "code",
            client: None,
            iteration,
        };
        prepare_mpc_planner_turn(&params(Some(1))).expect("iteration 1 prepare");
        std::fs::write(&brief_path, "brief\n---\n").expect("simulate planner append");
        prepare_mpc_planner_turn(&params(Some(2))).expect("iteration 2 prepare");
        assert_eq!(
            std::fs::read_to_string(&brief_path).expect("read brief"),
            "brief\n"
        );
    });
}

#[test]
fn kiss_cov_mpc_planner_params_struct() {
    let _: Option<MpcPlannerParams> = None;
    let _: Option<super::MpcPlannerTurnPrepared> = None;
}

#[cfg(all(test, unix))]
#[path = "mpc_planner_tests_unix.rs"]
mod mpc_planner_tests_unix;
