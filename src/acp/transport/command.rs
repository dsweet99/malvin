use crate::acp::import_prelude::*;
use crate::acp::{effective_cursor_api_key, effective_cursor_auth_token};
// Build and spawn the `agent acp` child process.
//
// This file is `include!`d from `acp/mod.rs`. It does not declare its own `use std::path::Path`; it
// relies on the parent module’s imports. If you move or trim `mod.rs` `use` lines, restore `Path`
// (or add a local import) here so `BuildAgentAcpCommandArgs` keeps compiling.
use std::ffi::OsString;
use std::{io, process::Stdio};
use tokio::{process::{Child, Command}, time::sleep};

pub(crate) const AGENT_BIN: &str = "agent";

const PARENT_ENV_KEYS: &[&str] = &[
    "CURSOR_AUTH_TOKEN",
    "CURSOR_CONFIG_DIR",
    "HOME",
    "NO_OPEN_BROWSER",
    "XDG_CONFIG_HOME",
    "XDG_STATE_HOME",
];

/// Prepend common locations so `#!/usr/bin/env node` mock agents resolve when `PATH` is minimal.
pub(crate) fn prepend_standard_path_for_child(cmd: &mut Command) {
    const PREFIX: &str = "/usr/bin:/bin:/usr/local/bin";
    let merged = match std::env::var_os("PATH") {
        Some(p) if !p.is_empty() => {
            let mut o = OsString::from(PREFIX);
            o.push(":");
            o.push(p);
            o
        }
        _ => OsString::from(PREFIX),
    };
    cmd.env("PATH", merged);
}

pub(crate) fn forward_parent_env(cmd: &mut Command) {
    for &key in PARENT_ENV_KEYS {
        if let Ok(v) = std::env::var(key) {
            if !v.is_empty() {
                cmd.env(key, v);
            }
        }
    }
}

pub(crate) fn apply_api_and_auth(cmd: &mut Command, api_key: Option<&str>, auth_token: Option<&str>) {
    if let Some(k) = api_key {
        cmd.arg("--api-key").arg(k);
        cmd.env("CURSOR_API_KEY", k);
    }
    if let Some(t) = auth_token {
        cmd.arg("--auth-token").arg(t);
        cmd.env("CURSOR_AUTH_TOKEN", t);
    }
}

pub(crate) fn apply_acp_tail(cmd: &mut Command, cwd: &Path, george_acp_lane: Option<&str>) {
    cmd.arg("acp");
    cmd.env("MALVIN_WORKSPACE", cwd);
    if let Some(lane) = george_acp_lane.map(str::trim).filter(|s| !s.is_empty()) {
        cmd.env("GEORGE_ACP_LANE", lane);
    }
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .current_dir(cwd);
}

/// Arguments for [`build_agent_acp_command`].
pub(crate) struct BuildAgentAcpCommandArgs<'a> {
    pub cwd: &'a Path,
    pub bin_override: Option<&'a Path>,
    pub api_key: Option<&'a str>,
    pub auth_token: Option<&'a str>,
    pub george_acp_lane: Option<&'a str>,
    pub model: Option<&'a str>,
    pub force: bool,
    pub sandbox: bool,
}

pub(crate) fn agent_program(bin_override: Option<&Path>) -> String {
    bin_override
        .map(|p| p.to_string_lossy().into_owned())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| AGENT_BIN.to_string())
}

pub(crate) fn build_agent_acp_command(args: &BuildAgentAcpCommandArgs<'_>) -> Command {
    let mut cmd = Command::new(agent_program(args.bin_override));
    forward_parent_env(&mut cmd);
    let api_key = effective_cursor_api_key(args.api_key);
    let auth_token = effective_cursor_auth_token(args.auth_token);
    apply_api_and_auth(&mut cmd, api_key.as_deref(), auth_token.as_deref());
    if args.force {
        cmd.arg("--force");
    }
    if args.sandbox {
        cmd.arg("--sandbox").arg("enabled");
    }
    if let Some(m) = args.model.map(str::trim).filter(|s| !s.is_empty()) {
        cmd.arg("--model").arg(m);
    }
    apply_acp_tail(&mut cmd, args.cwd, args.george_acp_lane);
    prepend_standard_path_for_child(&mut cmd);
    isolate_agent_process_group(&mut cmd);
    cmd
}

#[cfg(unix)]
fn isolate_agent_process_group(cmd: &mut Command) {
    cmd.process_group(0);
}

#[cfg(not(unix))]
fn isolate_agent_process_group(_: &mut Command) {}

pub(crate) async fn spawn_agent_acp_child(cmd: &mut Command) -> Result<Child, io::Error> {
    const ATTEMPTS: u32 = 16;
    const DELAY_MS: u64 = 10;
    for attempt in 0..ATTEMPTS {
        match cmd.spawn() {
            Ok(child) => return Ok(child),
            Err(e) if executable_text_busy(&e) && attempt + 1 < ATTEMPTS => {
                sleep(std::time::Duration::from_millis(DELAY_MS)).await;
            }
            Err(e) => return Err(e),
        }
    }
    Err(io::Error::other(
        "agent acp spawn retries exhausted (internal error)",
    ))
}

pub(crate) fn executable_text_busy(err: &io::Error) -> bool {
    if err.kind() == io::ErrorKind::ExecutableFileBusy {
        return true;
    }
    #[cfg(unix)]
    {
        err.raw_os_error() == Some(26)
    }
    #[cfg(not(unix))]
    {
        let _ = err;
        false
    }
}

#[test]
fn build_agent_acp_command_uses_bin_override_program() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(Path::new("/bin/true")),
        api_key: None,
        auth_token: None,
        george_acp_lane: None,
        model: None,
        force: false,
        sandbox: false,
    });
    assert_eq!(
        cmd.as_std().get_program().to_string_lossy(),
        "/bin/true"
    );
}

#[test]
fn transport_command_kiss_static_refs() {
    let _ = prepend_standard_path_for_child;
    let _ = forward_parent_env;
    let _ = apply_api_and_auth;
    let _ = apply_acp_tail;
    let _ = agent_program;
    let _ = isolate_agent_process_group;
    let _ = spawn_agent_acp_child;
    let _ = executable_text_busy;
}

