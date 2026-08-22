use std::collections::HashSet;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use pi::sdk::{SessionOptions, ThinkingLevel};

use crate::acp::AgentError;
use crate::agent_backend::SdkSession;
use crate::bridge_sdk::{BridgeSpawnArgs, StreamLog};

use super::isolated_bash::isolated_tool_factory;
use super::runtime::PiRuntime;
use super::session::PiEmbeddedSession;

type SandboxBaseline = HashSet<u32>;

fn sandbox_note_or_error(cwd: &Path) -> Result<SandboxBaseline, AgentError> {
    let baseline = crate::malvin_sandbox::malvin_spawn_baseline();
    crate::malvin_sandbox::note_active_sandbox_session(None, baseline.clone(), cwd)
        .map_err(AgentError)?;
    Ok(baseline)
}

pub(crate) async fn pi_spawn_bridge(args: BridgeSpawnArgs<'_>) -> Result<SdkSession, AgentError> {
    if !args.io.force {
        return Err(AgentError(crate::acp::NO_FORCE_MSG.into()));
    }
    crate::malvin_sandbox::assert_dead_before_next_spawn().map_err(AgentError)?;
    let (provider, model) = split_provider_model(args.model)?;
    if test_no_real_agent() {
        return Ok(SdkSession::Pi(Box::new(fake_embedded_session(&args))));
    }
    let options = build_session_options(&args, provider, model)?;
    let runtime = PiRuntime::start(options).map_err(AgentError)?;
    let session = embedded_session(&args, runtime)?;
    start_embedded_mem_watch(&session);
    Ok(SdkSession::Pi(Box::new(session)))
}

fn test_no_real_agent() -> bool {
    crate::acp::test_no_real_agent_enabled()
}

fn pi_thinking_level(thinking: &str) -> Result<ThinkingLevel, String> {
    let mapped = match thinking {
        "ultra" => "max",
        other => other,
    };
    ThinkingLevel::from_str(mapped)
}

fn build_session_options(
    args: &BridgeSpawnArgs<'_>,
    provider: &str,
    model: &str,
) -> Result<SessionOptions, AgentError> {
    let thinking = args
        .thinking
        .map(pi_thinking_level)
        .transpose()
        .map_err(AgentError)?;
    Ok(SessionOptions {
        provider: Some(provider.to_string()),
        model: Some(model.to_string()),
        thinking,
        working_directory: Some(args.cwd.to_path_buf()),
        no_session: true,
        extension_paths: Vec::new(),
        tool_factory: Some(isolated_tool_factory()),
        ..SessionOptions::default()
    })
}

fn fake_embedded_session(args: &BridgeSpawnArgs<'_>) -> PiEmbeddedSession {
    let baseline = crate::malvin_sandbox::malvin_spawn_baseline();
    note_sandbox_baseline(None, baseline.clone(), args.cwd);
    PiEmbeddedSession {
        runtime: None,
        log: StreamLog::from_spawn(args),
        work_dir: args.cwd.to_path_buf(),
        reader_dead: Arc::new(AtomicBool::new(false)),
        spawn_pid_baseline: baseline,
    }
}

fn embedded_session(
    args: &BridgeSpawnArgs<'_>,
    runtime: PiRuntime,
) -> Result<PiEmbeddedSession, AgentError> {
    let baseline = sandbox_note_or_error(args.cwd)?;
    Ok(PiEmbeddedSession {
        runtime: Some(runtime),
        log: StreamLog::from_spawn(args),
        work_dir: args.cwd.to_path_buf(),
        reader_dead: Arc::new(AtomicBool::new(false)),
        spawn_pid_baseline: baseline,
    })
}

fn note_sandbox_baseline(pgid: Option<u32>, baseline: SandboxBaseline, cwd: &Path) {
    let _ = crate::malvin_sandbox::note_active_sandbox_session(pgid, baseline.clone(), cwd);
}

pub(crate) fn split_provider_model(slug: &str) -> Result<(&str, &str), AgentError> {
    let Some((provider, model)) = slug.split_once('/') else {
        return Err(AgentError(format!(
            "pi model id must be `pi:<provider>/<model>` (got slug `{slug}`)"
        )));
    };
    if provider.is_empty() || model.is_empty() {
        return Err(AgentError(format!(
            "pi model id must be `pi:<provider>/<model>` (got slug `{slug}`)"
        )));
    }
    Ok((provider, model))
}

fn start_embedded_mem_watch(session: &PiEmbeddedSession) {
    #[cfg(unix)]
    {
        if crate::acp::test_no_real_agent_enabled() {
            return;
        }
        let reader_dead = Arc::clone(&session.reader_dead);
        let baseline = session.spawn_pid_baseline.clone();
        let work_dir = session.work_dir.clone();
        let run_dir = session.log.run_dir.clone();
        tokio::spawn(async move {
            watch_embedded_memory(reader_dead, baseline, work_dir, run_dir).await;
        });
    }
    #[cfg(not(unix))]
    {
        let _ = session;
    }
}

#[cfg(unix)]
async fn watch_embedded_memory(
    reader_dead: Arc<AtomicBool>,
    baseline: std::collections::HashSet<u32>,
    work_dir: std::path::PathBuf,
    run_dir: Option<std::path::PathBuf>,
) {
    let limit_bytes = crate::mem_limit_config::load_mem_limit_bytes(&work_dir);
    crate::acp::watch_process_group_memory(crate::acp::MemWatchHandles {
        reader_dead,
        pgid: None,
        limit_bytes,
        spawn_pid_baseline: baseline,
        run_dir,
    })
    .await;
}

#[cfg(test)]
mod thinking_arg_tests {
    use super::split_provider_model;

    #[test]
    fn split_keeps_model_path_after_first_slash() {
        assert_eq!(
            split_provider_model("openai/gpt-5").expect("ok"),
            ("openai", "gpt-5")
        );
    }
}
