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
use super::openrouter_pricing;
use super::runtime::PiRuntime;
use super::session::PiEmbeddedSession;

type SandboxBaseline = HashSet<u32>;

fn sandbox_note_or_error(
    ticket: crate::malvin_sandbox::SandboxSpawnTicket,
    cwd: &Path,
) -> Result<SandboxBaseline, AgentError> {
    let baseline = crate::malvin_sandbox::malvin_spawn_baseline();
    crate::malvin_sandbox::note_active_sandbox_session(ticket, None, baseline.clone(), cwd)
        .map_err(AgentError)?;
    Ok(baseline)
}

fn prewarm_openrouter_pricing(provider: &str) {
    if provider.eq_ignore_ascii_case("openrouter") {
        openrouter_pricing::warm_openrouter_pricing_cache(false);
    }
}

fn spawn_live_pi_bridge(
    ticket: crate::malvin_sandbox::SandboxSpawnTicket,
    args: &BridgeSpawnArgs<'_>,
    provider: &str,
    model: &str,
) -> Result<SdkSession, AgentError> {
    prewarm_openrouter_pricing(provider);
    let options = build_session_options(args, provider, model)?;
    let runtime = PiRuntime::start(options).map_err(AgentError)?;
    let session = embedded_session(ticket, args, runtime, (provider, model))?;
    super::session_spawn_watch::start_embedded_mem_watch(&session);
    Ok(SdkSession::Pi(Box::new(session)))
}

pub(crate) async fn pi_spawn_bridge(args: BridgeSpawnArgs<'_>) -> Result<SdkSession, AgentError> {
    let ticket = crate::malvin_sandbox::take_sandbox_spawn_ticket().map_err(AgentError)?;
    let (provider, model) = args.model.pi_provider_and_model().ok_or_else(|| {
        AgentError(format!(
            "rpi model id must be `rpi:<provider>/<model>` (got `{}`)",
            args.model.canonical()
        ))
    })?;
    if test_no_real_agent() {
        return Ok(SdkSession::Pi(Box::new(fake_embedded_session(
            ticket, &args, provider, model,
        ))));
    }
    spawn_live_pi_bridge(ticket, &args, provider, model)
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

const LOCAL_ENABLED_TOOLS: &[&str] = &["read", "bash", "edit", "write", "grep", "find", "ls"];

pub(crate) fn local_append_system_prompt(keyless: bool) -> Option<String> {
    keyless.then(|| {
        concat!(
            "You are a non-interactive CLI agent. For any shell/file action emit ONLY ",
            "a JSON tool call {\"name\":\"bash\",\"parameters\":{\"command\":\"...\"}} ",
            "(or read/write/edit/grep/find/ls). Never invent results. Never answer with ",
            "markdown ```bash fences. Stay in the workspace unless a temp path is named. ",
            "For HTTP(S) downloads prefer curl -L -o FILE URL. ",
            "To count files prefer bash with find . -type f | wc -l; do not install packages."
        )
        .to_string()
    })
}

pub(crate) fn local_enabled_tools(keyless: bool) -> Option<Vec<String>> {
    keyless.then(|| {
        LOCAL_ENABLED_TOOLS
            .iter()
            .map(|s| (*s).to_string())
            .collect()
    })
}

fn local_max_tool_iterations(keyless: bool) -> usize {
    if keyless {
        40
    } else {
        SessionOptions::default().max_tool_iterations
    }
}

fn ensure_local_catalog(
    cwd: &std::path::Path,
    provider: &str,
    model: &str,
) -> Result<(), AgentError> {
    let context_size = super::local_context::context_size_for_workdir(cwd);
    super::local_context::ensure_capped_local_model_catalog(provider, model, context_size)
        .map_err(AgentError)?;
    if pi::provider_metadata::provider_is_keyless_local(provider) {
        super::local_lifecycle::ensure_local_llm(provider, model).map_err(AgentError)?;
    }
    Ok(())
}

fn local_session_overrides(keyless: bool) -> (Option<String>, Option<Vec<String>>, usize) {
    (
        local_append_system_prompt(keyless),
        local_enabled_tools(keyless),
        local_max_tool_iterations(keyless),
    )
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
    ensure_local_catalog(args.cwd, provider, model)?;
    let keyless = pi::provider_metadata::provider_is_keyless_local(provider);
    let (append_system_prompt, enabled_tools, max_tool_iterations) =
        local_session_overrides(keyless);
    Ok(SessionOptions {
        provider: Some(provider.to_string()),
        model: Some(model.to_string()),
        api_key: keyless.then(|| super::local_context::KEYLESS_LOCAL_API_KEY.to_string()),
        thinking,
        append_system_prompt,
        enabled_tools,
        working_directory: Some(args.cwd.to_path_buf()),
        no_session: true,
        extension_paths: Vec::new(),
        tool_factory: Some(isolated_tool_factory()),
        max_tool_iterations,
        ..SessionOptions::default()
    })
}

fn fake_embedded_session(
    ticket: crate::malvin_sandbox::SandboxSpawnTicket,
    args: &BridgeSpawnArgs<'_>,
    provider: &str,
    model: &str,
) -> PiEmbeddedSession {
    let baseline = crate::malvin_sandbox::malvin_spawn_baseline();
    note_sandbox_baseline(ticket, None, baseline.clone(), args.cwd);
    PiEmbeddedSession {
        runtime: None,
        log: StreamLog::from_spawn(args),
        work_dir: args.cwd.to_path_buf(),
        reader_dead: Arc::new(AtomicBool::new(false)),
        spawn_pid_baseline: baseline,
        pi_provider: provider.to_string(),
        pi_model: model.to_string(),
        local_hold: take_local_hold(provider).unwrap_or(false),
    }
}

fn embedded_session(
    ticket: crate::malvin_sandbox::SandboxSpawnTicket,
    args: &BridgeSpawnArgs<'_>,
    runtime: PiRuntime,
    model_id: (&str, &str),
) -> Result<PiEmbeddedSession, AgentError> {
    let (provider, model) = model_id;
    let baseline = sandbox_note_or_error(ticket, args.cwd)?;
    let local_hold = take_local_hold(provider)?;
    Ok(PiEmbeddedSession {
        runtime: Some(runtime),
        log: StreamLog::from_spawn(args),
        work_dir: args.cwd.to_path_buf(),
        reader_dead: Arc::new(AtomicBool::new(false)),
        spawn_pid_baseline: baseline,
        pi_provider: provider.to_string(),
        pi_model: model.to_string(),
        local_hold,
    })
}

fn take_local_hold(provider: &str) -> Result<bool, AgentError> {
    if !pi::provider_metadata::provider_is_keyless_local(provider) {
        return Ok(false);
    }
    super::local_lifecycle::hold_local_llm().map_err(AgentError)?;
    Ok(true)
}

fn note_sandbox_baseline(
    ticket: crate::malvin_sandbox::SandboxSpawnTicket,
    pgid: Option<u32>,
    baseline: SandboxBaseline,
    cwd: &Path,
) {
    let _ = crate::malvin_sandbox::note_active_sandbox_session(ticket, pgid, baseline.clone(), cwd);
}
