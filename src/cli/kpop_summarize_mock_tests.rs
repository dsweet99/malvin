#![allow(unsafe_code)]

use crate::artifacts::create_kpop_run_artifacts;
use crate::cli::kpop_summarize::{
    kpop_outer_loop_summarize_params, run_outer_loop_summarize_if_warranted,
    KpopOuterLoopSummarizeInputs,
};
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
        let kpop_dir = artifacts.run_dir.join("_kpop");
        std::fs::create_dir_all(&kpop_dir).expect("mkdir");
        std::fs::write(kpop_dir.join("exp_log_test_g1.md"), "a\n").expect("write");
        std::fs::write(kpop_dir.join("exp_log_test_g2.md"), "b\n").expect("write");
        let shared = super::kpop_summarize_tests::summarize_shared_opts(DEFAULT_MAX_ACP_RETRIES);
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        rt.block_on(async {
            run_outer_loop_summarize_if_warranted(&kpop_outer_loop_summarize_params(
                KpopOuterLoopSummarizeInputs {
                    agent_ran: true,
                    shared: &shared,
                },
                store,
                artifacts,
            ))
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
