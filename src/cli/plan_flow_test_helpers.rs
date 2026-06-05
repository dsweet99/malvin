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
        gitignore: DotfileBackupState::Missing,
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
        client: crate::acp::AgentClient::with_max_acp_retries(
            "m".into(),
            crate::acp::AgentIoOptions {
                force: false,
                no_tee: true,
                raw_output: true,
                show_thoughts_on_stdout: false,
                emit_stdout_markdown: false,
                log_full_outgoing_prompts: false,
            },
            crate::support_paths::DEFAULT_MAX_ACP_RETRIES,
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

pub(super) fn post_1a_content(user: &str) -> String {
    format!("{user}\n\n---\nBEGIN_MALVIN\n## Restatement\nrestated\n")
}

pub(super) fn post_1b_content(user: &str) -> String {
    format!(
        "{}\n\n## Critique\ncrit\n\n## Open questions\n1. q?\n",
        post_1a_content(user).trim_end()
    )
}

pub(super) fn post_2_content(user: &str) -> String {
    format!(
        "{}\n\n## DECISIONS\n1. **Verdict:** ok **Evidence:** test\n",
        post_1b_content(user).trim_end()
    )
}

pub(super) fn plan_shared_opts_for_mock() -> crate::cli::SharedOpts {
    crate::cli::SharedOpts {
        model: crate::config::DEFAULT_CLI_MODEL.into(),
        no_force: true,
        no_tenacious: false,
        no_tee: true,
        no_markdown: true,
        verbose: false,
        max_acp_retries: crate::support_paths::DEFAULT_MAX_ACP_RETRIES,
        doc: false,
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

#[allow(unsafe_code)]
pub(super) fn install_plan_mock_env(mock: &Path, plan: &Path) {
    unsafe {
        std::env::set_var("MALVIN_AGENT_ACP_BIN", mock);
        std::env::set_var("CURSOR_AGENT_API_KEY", "test-key");
        std::env::set_var("MALVIN_TEST_PLAN_PATH", plan);
    }
}
