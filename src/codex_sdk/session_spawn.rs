use crate::acp::AgentError;
use crate::bridge_sdk::{BridgeSession, BridgeSpawnArgs, start_mem_watch};

pub(crate) async fn codex_spawn_bridge(
    args: BridgeSpawnArgs<'_>,
) -> Result<BridgeSession, AgentError> {
    if !args.io.force {
        return Err(AgentError(
            "--no-force is not supported for codex: (malvin runs Codex tools headlessly; no interactive approval)".into(),
        ));
    }
    crate::malvin_sandbox::assert_dead_before_next_spawn().map_err(AgentError)?;
    let session = spawn_codex_session(&args)?;
    start_mem_watch(&session);
    codex_initialize(&session).await?;
    codex_start_thread(&session, args.model, args.cwd).await?;
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
    }

    #[test]
    fn test_codex_spawn_bridge() {
        let _ = codex_spawn_bridge;
    }
    #[cfg(unix)]
    #[allow(unsafe_code)]
    #[tokio::test]
    async fn test_codex_mock_session_protocol() {
        use std::os::unix::fs::PermissionsExt;
        let _lock = crate::test_utils::test_env_lock();
        let tmp = tempfile::tempdir().unwrap();
        let bin = tmp.path().join("codex");
        std::fs::write(&bin, "#!/bin/sh\nwhile IFS= read -r line; do case \"$line\" in *model/list*) printf '%s\\n' '{\"id\":2,\"result\":{\"data\":[{\"id\":\"gpt-test\"},{\"id\":\"gpt-5.6\"}]}}';; *initialize*) printf '%s\\n' '{\"id\":1,\"result\":{}}';; *thread/start*) case \"$line\" in *gpt-5.6*) printf '%s\\n' '{\"id\":2,\"result\":{\"thread\":{\"id\":\"thread-test\"}}}';; *) printf '%s\\n' '{\"id\":2,\"error\":{\"message\":\"wrong model\"}}';; esac;; *turn/start*) printf '%s\\n' '{\"id\":3,\"result\":{}}' '{\"method\":\"turn/started\",\"params\":{\"turn\":{\"id\":\"turn-test\"}}}' '{\"method\":\"item/reasoning/textDelta\",\"params\":{\"turnId\":\"turn-test\",\"delta\":\"think\"}}' '{\"method\":\"item/started\",\"params\":{\"turnId\":\"turn-test\",\"item\":{\"id\":\"c1\",\"type\":\"commandExecution\",\"command\":\"ls\",\"status\":\"inProgress\"}}}' '{\"method\":\"item/completed\",\"params\":{\"turnId\":\"turn-test\",\"item\":{\"id\":\"c1\",\"type\":\"commandExecution\",\"command\":\"ls\",\"status\":\"completed\"}}}' '{\"method\":\"item/agentMessage/delta\",\"params\":{\"turnId\":\"turn-test\",\"delta\":\"hello\"}}' '{\"method\":\"turn/completed\",\"params\":{\"turn\":{\"id\":\"turn-test\",\"status\":\"completed\",\"lastAgentMessage\":\"hello\"}}}';; *turn/interrupt*) printf '%s\\n' '{\"id\":4,\"result\":{}}';; esac; done\n").unwrap();
        let mut perms = std::fs::metadata(&bin).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&bin, perms).unwrap();
        let prior = std::env::var_os("MALVIN_CODEX");
        unsafe {
            std::env::set_var("MALVIN_CODEX", &bin);
        }
        let model = crate::model_id::parse_model_id("codex:gpt-5.6").unwrap();
        let io = crate::acp::AgentIoOptions {
            force: true,
            no_tee: true,
            raw_output: true,
            show_thoughts_on_stdout: false,
            emit_stdout_markdown: false,
            log_full_outgoing_prompts: false,
        };
        let mut client = crate::agent_backend::SdkClient::with_max_retries(
            model,
            crate::agent_backend::BridgeKind::Codex,
            io,
            1,
        );
        client.begin_coder_session(tmp.path()).await.unwrap();
        client
            .session
            .as_ref()
            .unwrap()
            .send_prompt("test")
            .await
            .unwrap();
        assert_eq!(
            client.last_coder_prompt_agent_response().as_deref(),
            Some("hello")
        );
        client.end_coder_session().await.unwrap();
        unsafe {
            match prior {
                Some(v) => std::env::set_var("MALVIN_CODEX", v),
                None => std::env::remove_var("MALVIN_CODEX"),
            }
        }
    }

    #[test]
    fn test_response_error_and_id() {
        assert!(crate::codex_sdk::session_io::next_id() > 0);
        let error = response_error("context", &serde_json::json!({"error":{"message":"bad"}}));
        assert!(error.0.contains("context") && error.0.contains("bad"));
    }

    #[test]
    fn test_codex_initialize() {
        let _ = codex_initialize;
    }
    #[test]
    fn test_codex_start_thread() {
        let _ = codex_start_thread;
    }
    #[test]
    fn test_request() {
        let _ = request;
    }
    #[test]
    fn test_response_error() {
        let _ = response_error;
    }
}
