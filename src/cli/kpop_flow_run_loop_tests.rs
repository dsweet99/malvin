//! Tests for [`super::kpop_flow_run_loop`].

#[cfg(unix)]
mod unix_cov {
    #![allow(unsafe_code)]

    use super::super::kpop_flow_run_loop::{run_kpop_agent_loops, RunKpopAgentLoopsParams};
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;

    use crate::cli::kpop_flow::kpop_boot_store_client_prepared;
    use crate::cli::{KpopArgs, SharedOpts, WorkflowCliOptions};
    use crate::config::{DEFAULT_CLI_MODEL, DEFAULT_MAX_ACP_RETRIES};

    fn test_kpop_args(max_loops: usize) -> (KpopArgs, SharedOpts, WorkflowCliOptions) {
        let kpop = KpopArgs {
            max_loops,
            max_hypotheses: 10,
            tenacious: false,
            request: Some("investigate".into()),
        };
        let shared = SharedOpts {
            model: DEFAULT_CLI_MODEL.into(),
            no_force: true,
            no_tee: true,
            no_markdown: true,
            verbose: false,
            max_acp_retries: DEFAULT_MAX_ACP_RETRIES,
            doc: false,
        };
        let workflow = WorkflowCliOptions { force: false };
        (kpop, shared, workflow)
    }

    fn install_mock_agent_env(workspace: &Path, mock: &Path) {
        write_mock_agent(mock);
        let bin_dir = workspace.join("bin");
        std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
        crate::test_agent_client::install_exit_gate_bin(&bin_dir, "kiss", 0);
        let path = format!(
            "{}:{}",
            bin_dir.display(),
            std::env::var("PATH").unwrap_or_default()
        );
        unsafe {
            std::env::set_var("MALVIN_AGENT_ACP_BIN", mock);
            std::env::set_var("PATH", path);
            std::env::set_var("CURSOR_AGENT_API_KEY", "test-key");
        }
    }

    fn write_mock_agent(path: &Path) {
        let body = r"    const fs = require('fs');
    const path = require('path');
    const promptText = (((msg.params || {}).prompt || [])[0] || {}).text || '';
    const targetMatch = promptText.match(/exp_log_[^\s`]+\.md/);
    const target = targetMatch ? targetMatch[0] : null;
    const os = require('os');
    const root = path.join(os.homedir(), '.malvin', 'logs');
    if (target && fs.existsSync(root)) {
      outer: for (const hash of fs.readdirSync(root, { withFileTypes: true }).filter((e) => e.isDirectory())) {
        const bucket = path.join(root, hash.name);
        const runs = fs.readdirSync(bucket, { withFileTypes: true })
          .filter((e) => e.isDirectory()).map((e) => e.name).sort().reverse();
        for (const run of runs) {
          const p = path.join(bucket, run, '_kpop', target);
          if (fs.existsSync(p)) {
            fs.appendFileSync(p, `\n## Step 1 — KPOP mock\n`);
            break outer;
          }
        }
      }
    }";
        let handler = format!(
            "{body}\n    console.log(JSON.stringify({{ jsonrpc: '2.0', method: 'session/update', params: {{ update: {{ sessionUpdate: 'agent_message_chunk', content: {{ type: 'text', text: 'step\\n' }} }} }} }}));"
        );
        let script = format!("#!/usr/bin/env node\n{}\n", crate::acp_mock_js("", &handler));
        std::fs::write(path, script).expect("write mock");
        let mut perms = std::fs::metadata(path).expect("meta").permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(path, perms).expect("chmod");
    }

    #[test]
    fn run_kpop_agent_loops_propagates_exp_log_setup_error() {
        crate::test_utils::with_isolated_home(|workspace| {
            let rt = tokio::runtime::Runtime::new().expect("runtime");
            rt.block_on(async {
                std::fs::write(workspace.join(".kissconfig"), "k = 1\n").expect("kissconfig");
                let (kpop, shared, workflow) = test_kpop_args(2);
                let (store, mut client, prepared) =
                    kpop_boot_store_client_prepared(&kpop, &shared, workflow).expect("boot");
                let kpop_dir = prepared.artifacts.run_dir.join("_kpop");
                std::fs::remove_dir_all(&kpop_dir).expect("rm _kpop");
                std::fs::write(&kpop_dir, "not a directory").expect("block _kpop");
                let outcome = run_kpop_agent_loops(RunKpopAgentLoopsParams {
                    kpop: &kpop,
                    store: &store,
                    client: &mut client,
                    prepared: &prepared,
                })
                .await;
                assert!(outcome.agent_ran);
                outcome.acp_result.expect_err("exp log setup");
            });
        });
    }

    #[test]
    fn run_kpop_agent_loops_executes_mock_agent() {
        crate::test_utils::with_isolated_home(|workspace| {
            let rt = tokio::runtime::Runtime::new().expect("runtime");
            rt.block_on(async {
                std::fs::write(workspace.join(".kissconfig"), "k = 1\n").expect("kissconfig");
                let mock = workspace.join("mock-agent");
                install_mock_agent_env(workspace, &mock);
                let (kpop, shared, workflow) = test_kpop_args(1);
                let (store, mut client, prepared) =
                    kpop_boot_store_client_prepared(&kpop, &shared, workflow).expect("boot");
                let outcome = run_kpop_agent_loops(RunKpopAgentLoopsParams {
                    kpop: &kpop,
                    store: &store,
                    client: &mut client,
                    prepared: &prepared,
                })
                .await;
                assert!(outcome.agent_ran, "mock agent loop should set agent_ran");
                outcome.acp_result.expect("loops");
                let text =
                    std::fs::read_to_string(prepared.artifacts.exp_log_path()).expect("read");
                assert!(text.contains("## Step 1 — KPOP mock"));
            });
        });
    }
}
