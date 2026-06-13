#![allow(unsafe_code)]

use crate::artifacts::create_kpop_run_artifacts;
use crate::cli::kpop_summarize::run_inline_summarize_coder_prompt;
use crate::config::DEFAULT_MAX_ACP_RETRIES;
use crate::prompts::PromptStore;

pub(crate) fn write_mock_summarize_agent(path: &std::path::Path) {
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

pub(crate) fn with_summarize_mock_agent<F>(f: F)
where
    F: FnOnce(&std::path::Path, &PromptStore, &crate::artifacts::RunArtifacts),
{
    crate::test_utils::enable_test_fast_teardown();
    crate::test_utils::with_isolated_home(|workspace| {
        std::fs::create_dir_all(workspace.join(".malvin")).expect("mkdir");
        let artifacts = create_kpop_run_artifacts("kpop", Some(workspace)).expect("artifacts");
        let store = PromptStore::default_store();
        store.ensure_defaults().expect("defaults");
        let mock = workspace.join("mock-summarize-agent");
        write_mock_summarize_agent(&mock);
        unsafe {
            std::env::set_var(crate::acp::MALVIN_TEST_NO_REAL_AGENT_ENV, "1");
            std::env::set_var("MALVIN_AGENT_ACP_BIN", &mock);
            std::env::set_var("CURSOR_AGENT_API_KEY", "test-key");
        }
        f(workspace, &store, &artifacts);
    });
}

fn write_summarize_fixture_exp_logs(artifacts: &crate::artifacts::RunArtifacts) {
    let kpop_dir = artifacts.run_dir.join("_kpop");
    std::fs::create_dir_all(&kpop_dir).expect("mkdir");
    std::fs::write(kpop_dir.join("exp_log_test_g1.md"), "a\n").expect("write");
    std::fs::write(kpop_dir.join("exp_log_test_g2.md"), "b\n").expect("write");
}

async fn run_inline_summarize_on_open_mock_session(
    shared: &crate::cli::SharedOpts,
    store: &PromptStore,
    artifacts: &crate::artifacts::RunArtifacts,
) -> Result<(), String> {
    let mut client = crate::agent_backend::build_agent_backend(
        shared,
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
    run_inline_summarize_coder_prompt(&mut client, store, artifacts, "malvin kpop").await?;
    client.end_coder_session().await.map_err(|e| e.to_string())?;
    Ok(())
}

#[test]
fn run_inline_summarize_coder_prompt_runs_on_open_session() {
    with_summarize_mock_agent(|workspace, store, artifacts| {
        write_summarize_fixture_exp_logs(artifacts);
        let shared = super::kpop_summarize_tests::summarize_shared_opts(DEFAULT_MAX_ACP_RETRIES);
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        rt.block_on(async {
            run_inline_summarize_on_open_mock_session(&shared, store, artifacts)
                .await
                .expect("inline summarize");
        });
        let probe = workspace.join("summary_probe.log");
        assert!(probe.is_file(), "inline summarize should run on open session");
        let text = std::fs::read_to_string(probe).expect("read probe");
        assert!(text.contains("Summarize the activity"));
        assert!(text.contains("Executive summary"));
        assert!(artifacts.log_path("summary").is_file());
    });
}
