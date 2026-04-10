use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;

use crate::acp::{AcpSession, AcpSpawnArgs};
use crate::review_sync::{is_lgtm, sync_review_file};

use super::pair::ReviewerPromptPair;
use super::AgentError;
use super::client::AgentClient;

pub(super) fn has_api_key() -> bool {
    for key in ["CURSOR_AGENT_API_KEY", "CURSOR_API_KEY", "AGENT_API_KEY"] {
        if std::env::var_os(key).is_some_and(|v| !v.is_empty()) {
            return true;
        }
    }
    false
}

pub(super) fn auth_probe(args: &[&str]) -> bool {
    StdCommand::new(args[0])
        .args(&args[1..])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub(super) async fn spawn_acp_session(client: &AgentClient, cwd: &Path) -> Result<AcpSession, AgentError> {
    let bin = std::env::var_os("MALVIN_AGENT_ACP_BIN").map(PathBuf::from);
    let rpc_secs = crate::config::acp_rpc_timeout_secs_from_env();
    let model = client.model.trim();
    let model_opt = (!model.is_empty()).then_some(model);
    AcpSession::spawn(AcpSpawnArgs {
        cwd,
        bin_override: bin.as_deref(),
        api_key: None,
        auth_token: None,
        rpc_timeout_secs: rpc_secs,
        acp_verbose: false,
        george_acp_lane: None,
        ui_idle_notify: None,
        model: model_opt,
        force: client.io.force,
    })
    .await
    .map_err(AgentError)
}

pub(super) async fn maybe_tee_log(client: &AgentClient, log_path: &Path) {
    if !client.io.tee && !client.io.tee_json {
        return;
    }
    let Ok(bytes) = tokio::fs::read(log_path).await else {
        return;
    };
    let text = String::from_utf8_lossy(&bytes);
    if client.io.tee_json {
        for line in text.lines() {
            println!("{line}");
        }
    } else {
        print!("{text}");
        if !text.ends_with('\n') {
            println!();
        }
    }
}

pub(super) async fn run_reviewer_pair_once(
    client: &AgentClient,
    pair: &ReviewerPromptPair<'_>,
) -> Result<(), AgentError> {
    let s = spawn_acp_session(client, pair.cwd).await?;

    let mut review_full = pair.review_body.to_string();
    if let Ok(style_text) = std::fs::read_to_string(&client.style_prompt_path) {
        let t = style_text.trim();
        if !t.is_empty() {
            review_full = format!("{t}\n\n{review_full}");
        }
    }

    s.prompt(&review_full, pair.review_log)
        .await
        .map_err(AgentError)?;
    maybe_tee_log(client, pair.review_log).await;

    sync_review_file(pair.workspace_review_path, pair.artifact_review_path);
    if is_lgtm(pair.artifact_review_path) {
        s.shutdown().await.map_err(AgentError)?;
        return Ok(());
    }

    s.prompt(pair.kpop_body, pair.kpop_log)
        .await
        .map_err(AgentError)?;
    maybe_tee_log(client, pair.kpop_log).await;

    s.shutdown().await.map_err(AgentError)?;
    Ok(())
}
