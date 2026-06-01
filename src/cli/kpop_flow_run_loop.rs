//! Outer `malvin kpop` agent loop (`--max-loops`, early exit on `## KPOP_SOLVED`).

mod kpop_flow_run_loop_types;
pub(crate) use kpop_flow_run_loop_types::RunKpopAgentLoopsParams;

use std::path::PathBuf;

use crate::artifacts::ensure_gate_exp_log_file;
use crate::kpop_progression::{agent_declared_success, read_exp_log_text, KpopMultiturnState};
use crate::kpop_multiturn_prompts::KpopMultiturnPrompts;
use crate::KpopTurnPrompts;

use super::kpop_flow_a::{kpop_run_acp_multiturn, KpopAcpMultiturnCtx};
use crate::cli::loop_opts::kpop_agent_loop_exp_iteration;
use crate::cli::workflow_kpop_shared::{
    effective_max_loops, gate_iteration_context, print_kpop_session_log_line,
};

pub(crate) async fn run_kpop_agent_loops(
    params: RunKpopAgentLoopsParams<'_>,
) -> Result<(), String> {
    let max_loops = effective_max_loops(params.kpop.max_loops);
    if max_loops > 1 {
        let legacy = params.prepared.artifacts.gate_exp_log_path(0);
        let _ = std::fs::remove_file(legacy);
    }
    let mut last_acp = Ok(());
    for agent_loop in 1..=max_loops {
        let exp_iter = kpop_agent_loop_exp_iteration(agent_loop, max_loops);
        let exp_log_path =
            ensure_gate_exp_log_file(&params.prepared.artifacts, exp_iter).map_err(|e| e.to_string())?;
        print_kpop_session_log_line(&params.prepared.artifacts, &exp_log_path);

        let iteration_context = gate_iteration_context(
            &params.prepared.context,
            &params.prepared.artifacts,
            &exp_log_path,
            exp_iter,
        );
        let builder = KpopMultiturnPrompts::Turn(KpopTurnPrompts {
            store: params.store,
            base: &iteration_context,
            request_text: &params.prepared.text,
            prepend_rules_once: agent_loop == 1,
        });
        let mut state =
            KpopMultiturnState::new(builder, exp_log_path.clone(), params.kpop.max_hypotheses)?;

        crate::gate_loop_session::set_active_gate_iteration(Some(exp_iter));
        last_acp = kpop_run_acp_multiturn(
            KpopAcpMultiturnCtx {
                client: params.client,
                prepared: params.prepared,
                state: &mut state,
            },
            if agent_loop == max_loops {
                crate::run_timing::acp_post_run::RunTimingSessionEnd::Finalize
            } else {
                crate::run_timing::acp_post_run::RunTimingSessionEnd::AccumulateRun
            },
        )
        .await;
        crate::gate_loop_session::set_active_gate_iteration(None);
        if last_acp.is_err() {
            break;
        }
        if kpop_exp_log_declares_solved(&exp_log_path)? {
            break;
        }
    }
    last_acp
}

fn kpop_exp_log_declares_solved(exp_log_path: &PathBuf) -> Result<bool, String> {
    let text = read_exp_log_text(exp_log_path)?;
    Ok(agent_declared_success(&text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kpop_exp_log_declares_solved_reads_marker() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("exp.md");
        std::fs::write(&path, "## KPOP_SOLVED\n").expect("write");
        assert!(kpop_exp_log_declares_solved(&path).expect("read"));
    }

    #[cfg(unix)]
    mod unix_cov {
        #![allow(unsafe_code)]

        use std::os::unix::fs::PermissionsExt;
        use std::path::Path;

        use crate::cli::kpop_flow::kpop_boot_store_client_prepared;
        use crate::cli::{KpopArgs, SharedOpts, WorkflowCliOptions};
        use crate::config::{DEFAULT_CLI_MODEL, DEFAULT_MAX_ACP_RETRIES};

        use super::{run_kpop_agent_loops, RunKpopAgentLoopsParams};

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
            let script =
                format!("#!/usr/bin/env node\n{}\n", crate::acp_mock_js("", &handler));
            std::fs::write(path, script).expect("write mock");
            let mut perms = std::fs::metadata(path).expect("meta").permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(path, perms).expect("chmod");
        }

        #[test]
        fn run_kpop_agent_loops_executes_mock_agent() {
            crate::test_utils::with_isolated_home(|workspace| {
                let rt = tokio::runtime::Runtime::new().expect("runtime");
                rt.block_on(async {
                    std::fs::write(workspace.join(".kissconfig"), "k = 1\n").expect("kissconfig");
                    let mock = workspace.join("mock-agent");
                    write_mock_agent(&mock);
                    let bin_dir = workspace.join("bin");
                    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
                    crate::test_agent_client::install_exit_gate_bin(&bin_dir, "kiss", 0);
                    let path = format!(
                        "{}:{}",
                        bin_dir.display(),
                        std::env::var("PATH").unwrap_or_default()
                    );
                    unsafe {
                        std::env::set_var("MALVIN_AGENT_ACP_BIN", &mock);
                        std::env::set_var("PATH", path);
                        std::env::set_var("CURSOR_AGENT_API_KEY", "test-key");
                    }
                    let kpop = KpopArgs {
                        max_loops: 1,
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
                    let (store, mut client, prepared) =
                        kpop_boot_store_client_prepared(&kpop, &shared, workflow).expect("boot");
                    run_kpop_agent_loops(RunKpopAgentLoopsParams {
                        kpop: &kpop,
                        store: &store,
                        client: &mut client,
                        prepared: &prepared,
                    })
                    .await
                    .expect("loops");
                    let text =
                        std::fs::read_to_string(prepared.artifacts.exp_log_path()).expect("read");
                    assert!(text.contains("## Step 1 — KPOP mock"));
                });
            });
        }
    }
}
