use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::plan_flow_pipeline::PlanRunPrep;
use super::plan_flow_prompt::{build_plan_render_context, prepare_plan_prompt_store};
use crate::artifacts::RunArtifacts;
use crate::session_dotfile_backup::DotfileBackupState;

pub(super) fn empty_session_dotfile_backups() -> crate::artifacts::SessionDotfileBackups {
    crate::artifacts::SessionDotfileBackups::from_parts(crate::artifacts::SessionDotfileParts {
        kissconfig: DotfileBackupState::Missing,
        malvin_checks: DotfileBackupState::Missing,
        kissignore: DotfileBackupState::Missing,
        malvin_config: DotfileBackupState::Missing,
        gitignore: crate::session_dotfile_backup::GitignoreBackup::Missing,
        vision: crate::session_dotfile_backup::VisionBackup::Missing,
        malvin_config_workspace: DotfileBackupState::Missing,
    })
}

pub(super) fn plan_flow_test_prep(tmp: &tempfile::TempDir) -> (RunArtifacts, PathBuf) {
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "# User\n").expect("write");
    let artifacts =
        crate::artifacts::create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
    (artifacts, plan)
}

pub(super) fn test_plan_run_prep(
    _tmp: &tempfile::TempDir,
    artifacts: &RunArtifacts,
    plan: &Path,
    render_ctx: HashMap<String, String>,
) -> PlanRunPrep {
    PlanRunPrep {
        client: crate::agent_backend::AgentBackend::Acp(
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
                crate::support_paths::DEFAULT_MAX_ACP_RETRIES.min(1),
            ),
        ),
        artifacts: artifacts.clone(),
        source_plan_path: plan.to_path_buf(),
        store: prepare_plan_prompt_store().expect("store"),
        render_ctx,
        session_dotfile_backups: empty_session_dotfile_backups(),
    }
}

pub(super) fn test_plan_run_prep_for_plan(
    tmp: &tempfile::TempDir,
    artifacts: &RunArtifacts,
    plan: &Path,
) -> PlanRunPrep {
    let render_ctx = build_plan_render_context(plan, tmp.path(), artifacts);
    test_plan_run_prep(tmp, artifacts, plan, render_ctx)
}

pub(super) fn post_1a_content() -> String {
    "## Restatement\nrestated\n".to_string()
}

pub(super) fn post_1b_content() -> String {
    "## Restatement\nrestated\n\n## Critique\ncrit\n\n## Open questions\n1. q?\n".to_string()
}

pub(super) fn post_2_content() -> String {
    "## Restatement\nrestated\n\n## Critique\ncrit\n\n## Open questions\n1. q?\n\n## DECISIONS\n1. **Verdict:** ok **Evidence:** test\n".to_string()
}

pub(super) fn plan_shared_opts_for_mock() -> crate::cli::SharedOpts {
    crate::cli::SharedOpts {
        model: crate::config::DEFAULT_CLI_MODEL.into(),
        no_force: true,
        no_tenacious: true,
        no_tee: true,
        no_markdown: true,
        verbose: false,
        max_acp_retries: 1,
        doc: false,
        name: None,
        mini: false,
        mini_max_bash_turns: 32,
    }
}

#[allow(clippy::needless_raw_string_hashes)]
pub(super) fn plan_pipeline_mock_handler_body() -> &'static str {
    r#"
    const fs = require('fs');
    const promptText = (((msg.params || {}).prompt || [])[0] || {}).text || '';
    const planPath = process.env.MALVIN_TEST_PLAN_PATH;
    if (!planPath) throw new Error('MALVIN_TEST_PLAN_PATH unset');
    if (promptText.includes('**Prompt 3**')) {
      console.log(JSON.stringify({ jsonrpc: '2.0', method: 'session/update', params: { update: { sessionUpdate: 'agent_message_chunk', content: { type: 'text', text: '```markdown\n# Revised\n\nDone.\n```' } } } }));
    } else if (promptText.includes('**Prompt 2**')) {
      fs.appendFileSync(planPath, '\n\n## DECISIONS\n1. **Verdict:** ok **Evidence:** mock\n');
    } else if (promptText.includes('**Prompt 1b**')) {
      fs.appendFileSync(planPath, '\n\n## Critique\ncrit\n\n## Open questions\n1. q?\n');
    } else if (promptText.includes('**Prompt 1a**')) {
      fs.appendFileSync(planPath, 'restated\n');
    }
"#
}

pub(super) fn write_plan_pipeline_mock_agent(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let handler = plan_pipeline_mock_handler_body();
    let script = format!("#!/usr/bin/env node\n{}", crate::acp_mock_js("", handler));
    std::fs::write(path, script).expect("write mock");
    let mut perms = std::fs::metadata(path).expect("meta").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).expect("chmod");
}

pub(super) fn plan_args_for_mock(plan: &Path) -> super::PlanArgs {
    super::PlanArgs {
        plan_path: plan.display().to_string(),
        out_path: "plan.md".to_string(),
    }
}

pub(super) async fn prepare_plan_mock_run(
    _work: &Path,
    mock: &Path,
    plan: &Path,
) -> super::plan_flow_pipeline::PlanRunPrep {
    write_plan_pipeline_mock_agent(mock);
    install_plan_mock_env(mock, plan);
    prepare_plan_mock_run_with_env(plan).await
}

pub(super) async fn prepare_plan_mock_run_with_env(plan: &Path) -> super::plan_flow_pipeline::PlanRunPrep {
    super::prepare_plan_run(
        &plan_args_for_mock(plan),
        &plan_shared_opts_for_mock(),
        crate::cli::WorkflowCliOptions { force: false },
    )
    .await
    .expect("prepare")
}

#[allow(unsafe_code)]
pub(super) fn install_plan_mock_env(mock: &Path, plan: &Path) {
    unsafe {
        std::env::set_var(crate::acp::MALVIN_TEST_NO_REAL_AGENT_ENV, "1");
        std::env::set_var("MALVIN_AGENT_ACP_BIN", mock);
        std::env::set_var("CURSOR_AGENT_API_KEY", "test-key");
        std::env::set_var("MALVIN_TEST_PLAN_PATH", plan);
    }
}
