//! Tests for [`super::kpop_flow_run_loop`].

#[cfg(unix)]
pub(crate) fn test_kpop_args(max_loops: usize) -> (crate::cli::KpopArgs, crate::cli::SharedOpts, crate::cli::WorkflowCliOptions) {
    use crate::cli::{KpopArgs, SharedOpts, WorkflowCliOptions};
    use crate::config::{DEFAULT_CLI_MODEL, DEFAULT_MAX_ACP_RETRIES};

    let kpop = KpopArgs {
        max_loops,
        max_hypotheses: 10,
        tenacious: false,
        request: Some("investigate".into()),
    };
    let shared = SharedOpts {
        model: DEFAULT_CLI_MODEL.into(),
        no_force: true,
        no_tenacious: false,
        no_tee: true,
        no_markdown: true,
        verbose: false,
        max_acp_retries: DEFAULT_MAX_ACP_RETRIES,
        doc: false,
    };
    let workflow = WorkflowCliOptions { force: false };
    (kpop, shared, workflow)
}

#[cfg(unix)]
pub(crate) fn install_mock_agent_env(workspace: &std::path::Path, mock: &std::path::Path) {
    #![allow(unsafe_code)]

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

#[cfg(unix)]
pub(crate) fn write_mock_agent(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

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

#[cfg(unix)]
mod unix_cov {
    use super::super::kpop_flow_run_loop::{run_kpop_agent_loops, RunKpopAgentLoopsParams};

    use crate::cli::kpop_flow::kpop_boot_store_client_prepared;

    use super::{install_mock_agent_env, test_kpop_args};

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
    fn kpop_outer_loop_resnapshots_each_agent_run() {
        crate::test_utils::with_isolated_home(|workspace| {
            let rt = tokio::runtime::Runtime::new().expect("runtime");
            rt.block_on(async {
                std::fs::write(workspace.join(".gitignore"), "baseline\n").expect("gitignore");
                let mock_body = r"    const fs = require('fs');
    const path = require('path');
    const outer = (typeof this.outerRuns === 'undefined') ? 0 : this.outerRuns;
    this.outerRuns = outer + 1;
    if (outer === 0) {
      fs.writeFileSync(path.join(process.cwd(), '.gitignore'), 'tampered\n', 'utf8');
    } else {
      const gi = fs.readFileSync(path.join(process.cwd(), '.gitignore'), 'utf8');
      if (gi !== 'baseline\n') {
        throw new Error('outer run 2 did not resnapshot gitignore');
      }
    }";
                let mock = workspace.join("mock-resnapshot");
                let handler = format!(
                    "{mock_body}\n    console.log(JSON.stringify({{ jsonrpc: '2.0', method: 'session/update', params: {{ update: {{ sessionUpdate: 'agent_message_chunk', content: {{ type: 'text', text: 'step\\n' }} }} }} }}));"
                );
                let script = format!(
                    "#!/usr/bin/env node\n{}\n",
                    crate::acp_mock_js("", &handler)
                );
                std::fs::write(&mock, script).expect("write mock");
                install_mock_agent_env(workspace, &mock);
                let (kpop, shared, workflow) = test_kpop_args(2);
                let (store, mut client, prepared) =
                    kpop_boot_store_client_prepared(&kpop, &shared, workflow).expect("boot");
                let outcome = run_kpop_agent_loops(RunKpopAgentLoopsParams {
                    kpop: &kpop,
                    store: &store,
                    client: &mut client,
                    prepared: &prepared,
                })
                .await;
                outcome.acp_result.expect("resnapshot outer loops");
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
