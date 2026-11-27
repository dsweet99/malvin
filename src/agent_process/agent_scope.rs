use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

use tracing::{info, warn};

use super::agent_cgroup::pid_cgroup_dir;
use super::agent_cgroup_nest::{
    LEAF_NAME, LEAF_SHIM_MARKER, OWN_GROUP_ARG, create_nested_group, delegates_memory,
    is_malvin_group_name, nested_group_name,
};
use super::malvin_sandbox::SandboxSpawnTicket;

pub const NO_CGROUP_ENV: &str = "MALVIN_NO_CGROUP";
const SYSTEMD_RUN: &str = "systemd-run";

static SCOPE_PROBE: OnceLock<ScopeSupport> = OnceLock::new();
static NEXT_SCOPE_SERIAL: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeTarget {
    Unit(String),
    Nested(PathBuf),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScopeSupport {
    WithOomPolicy,
    WithoutOomPolicy,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Enclosure {
    Outside,
    InsideMalvin,
    MalvinLeaf(PathBuf),
}

#[must_use]
pub(crate) fn scope_args(unit: &str, mem_limit_bytes: u64, oom_policy: bool) -> Vec<String> {
    let mut args: Vec<String> = vec![
        "--user".into(),
        "--scope".into(),
        "--quiet".into(),
        "--collect".into(),
        format!("--unit={unit}"),
        format!("--property=MemoryMax={mem_limit_bytes}"),
    ];
    if oom_policy {
        args.push("--property=OOMPolicy=continue".into());
    }
    args.push("--property=Delegate=yes".into());
    args.push("--".into());
    args
}

fn run_scope_probe(oom_policy: bool) -> Result<(), String> {
    let unit = next_scope_unit();
    let output = std::process::Command::new(SYSTEMD_RUN)
        .args(scope_args(&unit, 1 << 40, oom_policy))
        .arg("true")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .output()
        .map_err(|e| format!("cannot run {SYSTEMD_RUN}: {e}"))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format!(
        "{SYSTEMD_RUN} exited with {}: {}",
        output.status,
        stderr.trim()
    ))
}

fn probe_systemd_scope() -> ScopeSupport {
    classify_scope_probe(run_scope_probe)
}

pub(crate) fn classify_scope_probe(probe: impl Fn(bool) -> Result<(), String>) -> ScopeSupport {
    let Err(with_policy) = probe(true) else {
        info!("malvin agent sessions run in per-session systemd scopes (cgroup v2)");
        return ScopeSupport::WithOomPolicy;
    };
    warn!(
        error = %with_policy,
        "systemd-run rejected a scope with OOMPolicy=continue; retrying without OOMPolicy"
    );
    match probe(false) {
        Ok(()) => {
            info!(
                "malvin agent sessions run in per-session systemd scopes without OOMPolicy (cgroup v2)"
            );
            ScopeSupport::WithoutOomPolicy
        }
        Err(without_policy) => {
            warn!(
                error = %without_policy,
                "systemd user scopes unavailable; agent sessions use process-group containment"
            );
            ScopeSupport::Unavailable
        }
    }
}

fn scope_support() -> ScopeSupport {
    *SCOPE_PROBE.get_or_init(probe_systemd_scope)
}

fn cgroups_disabled() -> bool {
    cfg!(test)
        || !cfg!(target_os = "linux")
        || crate::env_flag::env_flag(NO_CGROUP_ENV)
        || crate::agent_process::test_no_real_agent_enabled()
}

#[must_use]
pub(crate) fn enclosure_of(own_dir: &Path) -> Enclosure {
    let named = |dir: &Path| {
        dir.file_name()
            .and_then(OsStr::to_str)
            .is_some_and(is_malvin_group_name)
    };
    if let Some(group) = own_dir.parent().filter(|group| named(group))
        && own_dir.file_name() == Some(OsStr::new(LEAF_NAME))
        && delegates_memory(group)
    {
        return Enclosure::MalvinLeaf(group.to_path_buf());
    }
    if own_dir.ancestors().any(named) {
        Enclosure::InsideMalvin
    } else {
        Enclosure::Outside
    }
}

fn own_enclosure() -> Enclosure {
    pid_cgroup_dir(std::process::id()).map_or(Enclosure::Outside, |dir| enclosure_of(&dir))
}

#[must_use]
pub fn agent_cgroups_enabled() -> bool {
    if cgroups_disabled() {
        return false;
    }
    match own_enclosure() {
        Enclosure::MalvinLeaf(_) => true,
        Enclosure::InsideMalvin => false,
        Enclosure::Outside => scope_support() != ScopeSupport::Unavailable,
    }
}

fn next_serial() -> u64 {
    NEXT_SCOPE_SERIAL.fetch_add(1, Ordering::Relaxed)
}

fn next_scope_unit() -> String {
    format!("malvin-agent-{}-{}", std::process::id(), next_serial())
}

fn nested_command(
    program: &OsStr,
    parent: &Path,
    limit: u64,
    ticket: &mut SandboxSpawnTicket,
) -> Option<std::process::Command> {
    let exe = crate::self_exe::self_exe().ok()?;
    let name = nested_group_name(std::process::id(), next_serial());
    let group = create_nested_group(parent, &name, limit)
        .map_err(
            |e| warn!(error = %e, parent = %parent.display(), "cannot create nested agent cgroup"),
        )
        .ok()?;
    let mut cmd = super::malvin_sandbox::malvin_std_command(exe);
    cmd.arg(LEAF_SHIM_MARKER).arg(&group).arg(program);
    ticket.set_scope_unit(Some(ScopeTarget::Nested(group)));
    Some(cmd)
}

fn scope_command(
    program: &OsStr,
    limit: u64,
    ticket: &mut SandboxSpawnTicket,
) -> std::process::Command {
    let unit = next_scope_unit();
    let mut cmd = super::malvin_sandbox::malvin_std_command(SYSTEMD_RUN);
    cmd.args(scope_args(
        &unit,
        limit,
        scope_support() == ScopeSupport::WithOomPolicy,
    ));
    if let Ok(exe) = crate::self_exe::self_exe() {
        cmd.arg(exe).arg(LEAF_SHIM_MARKER).arg(OWN_GROUP_ARG);
    }
    cmd.arg(program);
    ticket.set_scope_unit(Some(ScopeTarget::Unit(unit)));
    cmd
}

#[must_use]
pub fn malvin_agent_tokio_command(
    program: impl AsRef<OsStr>,
    work_dir: &Path,
    ticket: &mut SandboxSpawnTicket,
) -> tokio::process::Command {
    ticket.set_scope_unit(None);
    let program = program.as_ref();
    if !agent_cgroups_enabled() {
        return super::malvin_sandbox::malvin_tokio_command(program);
    }
    let limit = crate::config::mem_limit_config::load_mem_limit_bytes(work_dir);
    let cmd = match own_enclosure() {
        Enclosure::MalvinLeaf(parent) => nested_command(program, &parent, limit, ticket)
            .unwrap_or_else(|| super::malvin_sandbox::malvin_std_command(program)),
        Enclosure::InsideMalvin | Enclosure::Outside => scope_command(program, limit, ticket),
    };
    tokio::process::Command::from(cmd)
}

#[cfg(test)]
#[path = "agent_scope_tests.rs"]
mod agent_scope_tests;
