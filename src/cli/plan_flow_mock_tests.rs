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
fn plan_session_restores_gitignore_after_agent() {
    let _ = write_plan_gitignore_tamper_mock_agent;
    crate::test_utils::with_isolated_home(|work| {
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        rt.block_on(async {
            run_plan_gitignore_restore_case(work).await;
        });
    });
}

async fn run_plan_gitignore_restore_case(work: &std::path::Path) {
    let plan = work.join("plan.md");
    std::fs::write(&plan, "# User\n").expect("write");
    std::fs::write(work.join(".gitignore"), "plan-gitignore\n").expect("gitignore");
    let mock = work.join("mock-agent-plan-gitignore");
    write_plan_gitignore_tamper_mock_agent(&mock);
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
    crate::acp_post_run::merge_acp_with_workspace_session_restore_and_check_abort(
        Ok(()),
        &prep.artifacts.work_dir,
        &prep.session_dotfile_backups,
        &prep.artifacts.artifact_result_md(),
    )
    .expect("restore");
    assert_eq!(
        std::fs::read_to_string(work.join(".gitignore")).expect("read"),
        "plan-gitignore\n"
    );
}

fn write_plan_gitignore_tamper_mock_agent(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

    let tamper = r"    const path = require('path');
    fs.writeFileSync(path.join(process.cwd(), '.gitignore'), 'TAMPERED\n', 'utf8');";
    let body = super::plan_flow_test_helpers::plan_pipeline_mock_handler_body();
    let handler = body.replace(
        "    const planPath = process.env.MALVIN_TEST_PLAN_PATH;",
        &format!("    const planPath = process.env.MALVIN_TEST_PLAN_PATH;\n{tamper}"),
    );
    let script = format!("#!/usr/bin/env node\n{}\n", crate::acp_mock_js("", &handler));
    std::fs::write(path, script).expect("write mock");
    let mut perms = std::fs::metadata(path).expect("meta").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).expect("chmod");
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
