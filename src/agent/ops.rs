use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use std::time::Duration;

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

fn lookup_bin_on_path(bin: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(bin))
        .find(|candidate| candidate.is_file())
}

fn resolve_agent_bin() -> Option<PathBuf> {
    std::env::var_os("MALVIN_AGENT_ACP_BIN")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| lookup_bin_on_path("agent"))
        .or_else(|| lookup_bin_on_path("cursor-agent"))
}

pub(super) async fn spawn_acp_session(client: &AgentClient, cwd: &Path) -> Result<AcpSession, AgentError> {
    let bin = resolve_agent_bin();
    let rpc_secs = crate::config::acp_rpc_timeout_secs_from_env();
    let model = client.model.trim();
    let model_opt = (!model.is_empty()).then_some(model);
    AcpSession::spawn(AcpSpawnArgs {
        cwd,
        bin_override: bin.as_deref(),
        api_key: None,
        auth_token: None,
        rpc_timeout: Duration::from_secs(rpc_secs),
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
    if client.io.no_tee {
        return;
    }
    let Ok(bytes) = tokio::fs::read(log_path).await else {
        return;
    };
    let text = String::from_utf8_lossy(&bytes);
    print!("{text}");
    if !text.ends_with('\n') {
        println!();
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

pub(super) struct KpopFlowOnceArgs<'a> {
    pub cwd: &'a Path,
    pub kpop_prompt: &'a str,
    pub kpop_log: &'a Path,
    pub learn: Option<(&'a str, &'a Path)>,
}

pub(super) async fn run_kpop_flow_once(
    client: &AgentClient,
    args: KpopFlowOnceArgs<'_>,
) -> Result<(), AgentError> {
    let s = spawn_acp_session(client, args.cwd).await?;
    if let Err(e) = s.prompt(args.kpop_prompt, args.kpop_log).await {
        let _ = s.shutdown().await;
        return Err(AgentError(e));
    }
    maybe_tee_log(client, args.kpop_log).await;
    if let Some((learn_body, learn_log)) = args.learn {
        if let Err(e) = s.prompt(learn_body, learn_log).await {
            let _ = s.shutdown().await;
            return Err(AgentError(e));
        }
        maybe_tee_log(client, learn_log).await;
    }
    s.shutdown().await.map_err(AgentError)
}

#[cfg(test)]
mod tests {
    #![allow(unsafe_code)]

    use super::*;

    #[cfg(unix)]
    fn write_path_executable(path: &Path) {
        use std::os::unix::fs::PermissionsExt;

        std::fs::write(path, b"#!/bin/sh\nexit 0\n").unwrap();
        let mut perms = std::fs::metadata(path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(path, perms).unwrap();
        crate::test_utils::sync_test_executable(path);
    }

    #[test]
    #[cfg(unix)]
    fn resolve_agent_bin_prefers_env_override() {
        let _guard = crate::test_utils::test_env_lock();
        let tmp = tempfile::tempdir().unwrap();
        let override_bin = tmp.path().join("custom-agent");
        write_path_executable(&override_bin);
        let path_dir = tmp.path().join("path-bin");
        std::fs::create_dir(&path_dir).unwrap();
        write_path_executable(&path_dir.join("agent"));
        let old_override = std::env::var_os("MALVIN_AGENT_ACP_BIN");
        let old_path = std::env::var_os("PATH");

        unsafe {
            std::env::set_var("MALVIN_AGENT_ACP_BIN", &override_bin);
            std::env::set_var("PATH", &path_dir);
        }

        assert_eq!(resolve_agent_bin().as_deref(), Some(override_bin.as_path()));

        unsafe {
            if let Some(value) = old_override {
                std::env::set_var("MALVIN_AGENT_ACP_BIN", value);
            } else {
                std::env::remove_var("MALVIN_AGENT_ACP_BIN");
            }
            if let Some(value) = old_path {
                std::env::set_var("PATH", value);
            } else {
                std::env::remove_var("PATH");
            }
        }
    }

    #[test]
    #[cfg(unix)]
    fn resolve_agent_bin_falls_back_to_cursor_agent_on_path() {
        let _guard = crate::test_utils::test_env_lock();
        let tmp = tempfile::tempdir().unwrap();
        let path_dir = tmp.path().join("bin");
        std::fs::create_dir(&path_dir).unwrap();
        let cursor_agent = path_dir.join("cursor-agent");
        write_path_executable(&cursor_agent);
        let old_override = std::env::var_os("MALVIN_AGENT_ACP_BIN");
        let old_path = std::env::var_os("PATH");

        unsafe {
            std::env::remove_var("MALVIN_AGENT_ACP_BIN");
            std::env::set_var("PATH", &path_dir);
        }

        assert_eq!(resolve_agent_bin().as_deref(), Some(cursor_agent.as_path()));

        unsafe {
            if let Some(value) = old_override {
                std::env::set_var("MALVIN_AGENT_ACP_BIN", value);
            } else {
                std::env::remove_var("MALVIN_AGENT_ACP_BIN");
            }
            if let Some(value) = old_path {
                std::env::set_var("PATH", value);
            } else {
                std::env::remove_var("PATH");
            }
        }
    }
}
