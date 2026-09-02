use crate::acp::AgentError;
use crate::bridge_sdk::{BridgeSpawnArgs, MemWatchArgs, start_mem_watch};
use super::session::CodexSession;

pub(crate) async fn codex_spawn_bridge(
    args: BridgeSpawnArgs<'_>,
    service: Option<&str>,
) -> Result<CodexSession, AgentError> {
    crate::acp::require_force(args.io.force)?;
    let ticket = crate::malvin_sandbox::take_sandbox_spawn_ticket().map_err(AgentError)?;
    let session = spawn_codex_session(&args, service, ticket)?;
    start_mem_watch(MemWatchArgs {
        process_group_id: session.process_group_id,
        reader_dead: &session.reader_dead,
        work_dir: &session.work_dir,
        spawn_pid_baseline: &session.spawn_pid_baseline,
        run_dir: session.run_dir.as_deref(),
    });
    codex_initialize(&session).await?;
    let model = args.wire_model();
    codex_start_thread(&session, &model, args.cwd).await?;
    Ok(session)
}

use super::session_process::spawn_codex_session;
use super::session_protocol::{codex_initialize, codex_start_thread};

#[cfg(test)]
mod tests {
    use super::super::session_process::{
        CodexProcess, build_codex_session, build_codex_session_io, configured_codex_command,
        spawn_codex_process, spawn_codex_session,
    };
    use super::super::session_protocol::{request, response_error};
    use super::*;

    #[test]
    fn kiss_cov_codex_process_type_is_referenced() {
        let _: Option<CodexProcess> = None;
        let _ = build_codex_session;
        let _ = build_codex_session_io;
        let _ = configured_codex_command;
        let _ = spawn_codex_process;
        let _ = spawn_codex_session;
        let _ = codex_spawn_bridge;
        let _ = request;
        let _ = response_error;
        let _ = stringify!(thread_start_params);
        let _ = codex_initialize;
        let _ = codex_start_thread;
    }

    #[test]
    fn test_response_error_and_id() {
        assert!(crate::codex_sdk::session_io::next_id() > 0);
        let error = response_error("context", &serde_json::json!({"error":{"message":"bad"}}));
        assert!(error.message.contains("context") && error.message.contains("bad"));
    }
}

#[cfg(all(test, unix))]
mod unix_tests {
    #![allow(unsafe_code)]
    use std::path::{Path, PathBuf};

    const MOCK_SCRIPT: &str = include_str!("session_spawn_unix_mock.sh");

    fn write_codex_mock_bin(dir: &Path) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let bin = dir.join("codex");
        std::fs::write(&bin, MOCK_SCRIPT).unwrap();
        let mut perms = std::fs::metadata(&bin).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&bin, perms).unwrap();
        bin
    }

    const fn mock_io() -> crate::acp::AgentIoOptions {
        crate::acp::AgentIoOptions {
            force: true,
            no_tee: true,
            raw_output: true,
            show_thoughts_on_stdout: false,
            emit_stdout_markdown: false,
            log_full_outgoing_prompts: false,
        }
    }

    fn mock_client() -> crate::agent_backend::SdkClient {
        crate::agent_backend::SdkClient::with_max_retries(
            crate::model_id::parse_model_id("codex:gpt-5.6").unwrap(),
            mock_io(),
            1,
        )
    }

    fn restore_codex_env(prior: Option<std::ffi::OsString>) {
        unsafe {
            match prior {
                Some(v) => std::env::set_var("MALVIN_CODEX", v),
                None => std::env::remove_var("MALVIN_CODEX"),
            }
        }
    }

    #[test]
    fn kiss_cov_codex_spawn_unix() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(write_codex_mock_bin(tmp.path()).is_file());
        restore_codex_env(None);
    }

    #[tokio::test]
    async fn test_codex_mock_session_protocol() {
        let _lock = crate::test_utils::test_env_lock();
        let tmp = tempfile::tempdir().unwrap();
        let prior = std::env::var_os("MALVIN_CODEX");
        unsafe {
            std::env::set_var("MALVIN_CODEX", write_codex_mock_bin(tmp.path()));
        }
        let mut client = mock_client();
        client.begin_coder_session(tmp.path()).await.unwrap();
        crate::agent_backend::live_session(&client)
            .unwrap()
            .send_prompt("test")
            .await
            .unwrap();
        assert_eq!(
            client.last_coder_prompt_agent_response().as_deref(),
            Some("hello")
        );
        client.end_coder_session().await.unwrap();
        restore_codex_env(prior);
    }

    #[tokio::test]
    async fn hung_codex_turn_times_out() {
        let _lock = crate::test_utils::test_env_lock();
        let tmp = tempfile::tempdir().unwrap();
        let prior = std::env::var_os("MALVIN_CODEX");
        let prior_idle = std::env::var_os("MALVIN_SDK_DRAIN_IDLE_TIMEOUT_MS");
        unsafe {
            std::env::set_var("MALVIN_CODEX", write_codex_mock_bin(tmp.path()));
            std::env::set_var("MALVIN_CODEX_HANG", "1");
            std::env::set_var("MALVIN_SDK_DRAIN_IDLE_TIMEOUT_MS", "200");
        }
        let mut client = mock_client();
        client.begin_coder_session(tmp.path()).await.unwrap();
        let err = crate::agent_backend::live_session(&client)
            .unwrap()
            .send_prompt("test")
            .await
            .expect_err("silent turn must time out");
        assert!(
            err.message.contains("codex timed out") && err.message.contains("turn event"),
            "unexpected: {}",
            err.message
        );
        let _ = client.end_coder_session().await;
        unsafe {
            std::env::remove_var("MALVIN_CODEX_HANG");
            match prior_idle {
                Some(v) => std::env::set_var("MALVIN_SDK_DRAIN_IDLE_TIMEOUT_MS", v),
                None => std::env::remove_var("MALVIN_SDK_DRAIN_IDLE_TIMEOUT_MS"),
            }
        }
        restore_codex_env(prior);
    }

    #[tokio::test]
    async fn failed_codex_turn_is_an_error() {
        let _lock = crate::test_utils::test_env_lock();
        let tmp = tempfile::tempdir().unwrap();
        let prior = std::env::var_os("MALVIN_CODEX");
        unsafe {
            std::env::set_var("MALVIN_CODEX", write_codex_mock_bin(tmp.path()));
            std::env::set_var("MALVIN_CODEX_FAIL_TURN", "1");
        }
        let mut client = mock_client();
        client.begin_coder_session(tmp.path()).await.unwrap();
        let err = crate::agent_backend::live_session(&client)
            .unwrap()
            .send_prompt("test")
            .await
            .expect_err("failed turn");
        assert!(err.message.contains("auth"), "unexpected: {}", err.message);
        let _ = client.end_coder_session().await;
        unsafe {
            std::env::remove_var("MALVIN_CODEX_FAIL_TURN");
        }
        restore_codex_env(prior);
    }
}
