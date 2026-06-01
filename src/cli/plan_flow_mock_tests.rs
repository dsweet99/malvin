use super::plan_flow_pipeline::run_plan_acp;
use super::plan_flow_test_helpers::{
    install_plan_mock_env, plan_shared_opts_for_mock, write_plan_pipeline_mock_agent,
};
use super::{prepare_plan_run, PlanArgs};

#[test]
fn prepare_plan_run_truncates_and_loads_prompt_store() {
    crate::test_utils::with_isolated_home(|work| {
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        rt.block_on(async {
            let plan = work.join("plan.md");
            std::fs::write(&plan, "# User\n\n---\nBEGIN_MALVIN\nold\n").expect("write");
            let mock = work.join("mock-agent");
            write_plan_pipeline_mock_agent(&mock);
            install_plan_mock_env(&mock, &plan);
            let prep = prepare_plan_run(
                &PlanArgs {
                    plan_path: plan.display().to_string(),
                },
                &plan_shared_opts_for_mock(),
                crate::cli::WorkflowCliOptions { force: false },
            )
            .await
            .expect("prepare");
            assert_eq!(
                std::fs::read_to_string(&plan).expect("read"),
                "# User\n\n"
            );
            assert!(prep.store.validate_exists(crate::prompts::PLAN_1A_RESTATE_MD).is_ok());
        });
    });
}

#[test]
fn run_plan_acp_mock_agent_completes_four_prompt_pipeline() {
    crate::test_utils::with_isolated_home(|work| {
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        rt.block_on(async {
            let plan = work.join("plan.md");
            std::fs::write(&plan, "# User\n").expect("write");
            let mock = work.join("mock-agent");
            write_plan_pipeline_mock_agent(&mock);
            install_plan_mock_env(&mock, &plan);
            let mut prep = prepare_plan_run(
                &PlanArgs {
                    plan_path: plan.display().to_string(),
                },
                &plan_shared_opts_for_mock(),
                crate::cli::WorkflowCliOptions { force: false },
            )
            .await
            .expect("prepare");
            run_plan_acp(&mut prep).await.expect("plan acp");
            let out = std::fs::read_to_string(&plan).expect("read plan");
            assert!(out.contains("# Revised"));
            assert!(out.contains("---\nBEGIN_MALVIN"));
            assert!(prep.artifacts.run_dir.join("plan.p1a.md").is_file());
            assert!(prep.artifacts.run_dir.join("plan.p2.decisions.md").is_file());
        });
    });
}
