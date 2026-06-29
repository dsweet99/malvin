//! Unix mock-agent integration tests for MPC planner sessions.

use std::collections::HashMap;
use std::os::unix::fs::PermissionsExt;

use super::{run_mpc_planner_session, MpcPlannerParams};
use crate::cli::kpop_flow::kpop_flow_run_loop_tests::{install_mock_agent_env, test_kpop_args};
use super::super::mpc_planner_exp_log_path;
use crate::prompt_stratification::WorkflowRenderContext;
use crate::prompts::PromptStore;
use crate::test_utils::with_isolated_home;
use crate::workspace_paths::malvin_config_path;

macro_rules! mpc_append_mock_script {
    () => {
        format!(
            "#!/usr/bin/env node\n{}\n",
            crate::acp_mock_js(
                "",
                r"    const fs = require('fs');
    const path = require('path');
    const promptText = (((msg.params || {}).prompt || [])[0] || {}).text || '';
    const pathMatch = promptText.match(/`([^`]*user_request\.md)`/);
    if (pathMatch) {
      let p = pathMatch[1];
      if (p.startsWith('./')) p = path.join(process.cwd(), p.slice(2));
      else if (!path.isAbsolute(p)) p = path.join(process.cwd(), p);
      fs.appendFileSync(p, '---\n');
    }
    console.log(JSON.stringify({ jsonrpc: '2.0', method: 'session/update', params: { update: { sessionUpdate: 'agent_message_chunk', content: { type: 'text', text: 'mpc\n' } } } }));"
            )
        )
    };
}

macro_rules! mpc_enabled_workflow {
    ($work:expr) => {{
        let cfg_path = malvin_config_path($work);
        std::fs::create_dir_all(cfg_path.parent().expect("parent")).expect("mkdir");
        std::fs::write(&cfg_path, "mpc = true\n").expect("write mpc config");
        let mock = $work.join("mock-mpc-agent");
        let env = install_mock_agent_env($work, &mock);
        let script = mpc_append_mock_script!();
        std::fs::write(&mock, script.as_bytes()).expect("write mock");
        let mut perms = std::fs::metadata(&mock).expect("meta").permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&mock, perms).expect("chmod");
        let artifacts =
            crate::artifacts::create_kpop_run_artifacts("plan", Some($work)).expect("artifacts");
        let user_request = crate::artifacts::user_request_path(&artifacts);
        std::fs::write(&user_request, "brief\n").expect("write brief");
        let exp_log = mpc_planner_exp_log_path(&artifacts);
        std::fs::create_dir_all(exp_log.parent().expect("parent")).expect("mkdir _kpop");
        std::fs::write(&exp_log, "prior\n").expect("seed exp log");
        let mut context =
            crate::cli::workflow_kpop_shared::kpop_workflow_context(&artifacts, "code").expect("context");
        context.insert(
            "user_request_path".to_string(),
            crate::workflow_context::format_prompt_path(&user_request, &artifacts.work_dir),
        );
        let store = PromptStore::default_store();
        store.ensure_defaults().expect("defaults");
        (artifacts, context, store, user_request, env)
    }};
}

macro_rules! mpc_disabled_skip_workflow {
    ($work:expr) => {{
        crate::malvin_test_seed::seed_malvin_config($work, "mpc = false\n");
        let artifacts =
            crate::artifacts::create_kpop_run_artifacts("plan", Some($work)).expect("artifacts");
        let user_request = crate::artifacts::user_request_path(&artifacts);
        std::fs::write(&user_request, "brief\n").expect("write brief");
        let context = WorkflowRenderContext::from(HashMap::from([(
            "user_request_path".to_string(),
            "./user_request.md".to_string(),
        )]));
        let store = PromptStore::default_store();
        store.ensure_defaults().expect("defaults");
        (artifacts, context, store, user_request)
    }};
}

#[test]
fn run_mpc_planner_session_appends_separator_when_enabled() {
    with_isolated_home(|work| {
        crate::test_utils::block_on_test_async(async {
            let (artifacts, context, store, user_request, _env) = mpc_enabled_workflow!(work);
            let (_kpop, shared, workflow) = test_kpop_args(1);
            run_mpc_planner_session(MpcPlannerParams {
                shared: &shared,
                workflow,
                store: &store,
                artifacts: &artifacts,
                context: &context,
                command: "code",
                client: None,
                iteration: None,
            })
            .await
            .expect("mpc session");
            let text = std::fs::read_to_string(&user_request).expect("read brief");
            assert!(text.ends_with("---\n"), "got {text:?}");
            assert!(artifacts.log_path("mpc_planner").is_file());
        });
    });
}

#[test]
fn run_mpc_planner_session_reuses_existing_client() {
    with_isolated_home(|work| {
        crate::test_utils::block_on_test_async(async {
            let (artifacts, context, store, user_request, _env) = mpc_enabled_workflow!(work);
            let (_kpop, shared, workflow) = test_kpop_args(1);
            let mut client = crate::agent_backend::build_agent_backend(
                &shared,
                workflow,
                shared.acp_stdout_markdown_enabled(),
                "code",
            )
            .expect("backend");
            client.ensure_authenticated().expect("auth");
            run_mpc_planner_session(MpcPlannerParams {
                shared: &shared,
                workflow,
                store: &store,
                artifacts: &artifacts,
                context: &context,
                command: "code",
                client: Some(&mut client),
                iteration: None,
            })
            .await
            .expect("mpc with reused client");
            let text = std::fs::read_to_string(&user_request).expect("read brief");
            assert!(text.ends_with("---\n"));
        });
    });
}

#[test]
fn run_mpc_planner_session_skipped_when_disabled() {
    with_isolated_home(|work| {
        crate::test_utils::block_on_test_async(async {
            let (artifacts, context, store, user_request) = mpc_disabled_skip_workflow!(work);
            let (_kpop, shared, workflow) = test_kpop_args(1);
            run_mpc_planner_session(MpcPlannerParams {
                shared: &shared,
                workflow,
                store: &store,
                artifacts: &artifacts,
                context: &context,
                command: "code",
                client: None,
                iteration: None,
            })
            .await
            .expect("skip mpc");
            let text = std::fs::read_to_string(&user_request).expect("read brief");
            assert_eq!(text, "brief\n");
            assert!(!artifacts.log_path("mpc_planner").is_file());
        });
    });
}
