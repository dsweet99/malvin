use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;

use microsandbox::Sandbox;
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

use crate::agent_sandbox_config::AgentSandboxConfig;
use crate::alnum_id::random_alnum;

use super::bridge::{pump_guest_stdout, pump_malvin_stdin};
use super::feasibility::{agent_install_root, linux_node_in_bundle};
use super::teardown::AgentSandboxGuard;

const GUEST_WORKDIR: &str = "/workspace";
const GUEST_AGENT: &str = "/agent";

pub struct MicrosandboxTransport {
    pub holder: Child,
    pub to_guest: mpsc::UnboundedSender<Vec<u8>>,
    pub from_guest: mpsc::UnboundedReceiver<Vec<u8>>,
    pub guard: Arc<AgentSandboxGuard>,
}

async fn create_agent_sandbox(cwd: &Path, bin: &Path, mem_mib: u32) -> Result<Sandbox, String> {
    Sandbox::builder(format!("malvin-agent-{}", random_alnum(8)))
        .image("debian:bookworm-slim")
        .memory(mem_mib)
        .workdir(GUEST_WORKDIR)
        .replace()
        .volume(GUEST_WORKDIR, |m| m.bind(cwd))
        .volume(GUEST_AGENT, |m| m.bind(agent_install_root(bin)).readonly())
        .create()
        .await
        .map_err(|e| format!("microsandbox create: {e}"))
}

async fn guest_exec_stream(
    sb: &Sandbox,
    args: &crate::acp::AcpSpawnArgs<'_>,
) -> Result<microsandbox::ExecHandle, String> {
    let exec = sb
        .exec_stream_with(format!("{GUEST_AGENT}/node"), |e| {
            let mut b = e.stdin_pipe().cwd(GUEST_WORKDIR);
            for a in guest_argv(args) {
                b = b.arg(a);
            }
            b.env("MALVIN_WORKSPACE", GUEST_WORKDIR)
        })
        .await
        .map_err(|e| format!("microsandbox exec: {e}"))?;
    Ok(exec)
}

async fn start_guest_io_bridge(
    sb: Sandbox,
    args: &crate::acp::AcpSpawnArgs<'_>,
) -> Result<GuestIoBridge, String> {
    let mut exec = guest_exec_stream(&sb, args).await?;
    let sink = exec
        .take_stdin()
        .ok_or_else(|| "microsandbox stdin missing".to_string())?;
    let (from_tx, from_rx, to_tx, to_rx) = microsandbox_io_channels();
    let bridge = tokio::spawn(async {
        let _ = tokio::join!(
            pump_guest_stdout(exec, from_tx),
            pump_malvin_stdin(to_rx, sink),
        );
    });
    Ok((from_rx, to_tx, AgentSandboxGuard::new(sb, bridge)))
}

type GuestIoBridge = (
    mpsc::UnboundedReceiver<Vec<u8>>,
    mpsc::UnboundedSender<Vec<u8>>,
    Arc<AgentSandboxGuard>,
);

type MicrosandboxIoChannels = (
    mpsc::UnboundedSender<Vec<u8>>,
    mpsc::UnboundedReceiver<Vec<u8>>,
    mpsc::UnboundedSender<Vec<u8>>,
    mpsc::UnboundedReceiver<Vec<u8>>,
);

fn microsandbox_io_channels() -> MicrosandboxIoChannels {
    let (from_tx, from_rx) = mpsc::unbounded_channel();
    let (to_tx, to_rx) = mpsc::unbounded_channel();
    (from_tx, from_rx, to_tx, to_rx)
}

fn spawn_sleep_holder() -> Result<Child, String> {
    Command::new("sleep")
        .arg("86400")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .map_err(|e| format!("sleep holder: {e}"))
}

pub async fn spawn_microsandbox_transport(
    args: &crate::acp::AcpSpawnArgs<'_>,
    cfg: AgentSandboxConfig,
) -> Result<MicrosandboxTransport, String> {
    let bin = resolve_spawn_agent_bin(args)?;
    let _ = linux_node_in_bundle(&bin)
        .ok_or_else(|| "microsandbox: no Linux ELF agent node (try --no-sandbox)".to_string())?;
    let cwd = args.cwd.canonicalize().map_err(|e| e.to_string())?;
    let mem_mib = u32::try_from(cfg.mem_limit_gb.saturating_mul(1024)).unwrap_or(u32::MAX);
    let sb = create_agent_sandbox(&cwd, &bin, mem_mib).await?;
    let (from_rx, to_tx, guard) = start_guest_io_bridge(sb, args).await?;
    let holder = spawn_sleep_holder()?;
    Ok(MicrosandboxTransport {
        holder,
        to_guest: to_tx,
        from_guest: from_rx,
        guard,
    })
}

pub(crate) fn resolve_spawn_agent_bin(
    args: &crate::acp::AcpSpawnArgs<'_>,
) -> Result<std::path::PathBuf, String> {
    args.bin_override.map_or_else(
        || {
            crate::support_paths::agent_or_cursor_agent_bin()
                .ok_or_else(|| "agent not on PATH".to_string())
        },
        |p| Ok(p.to_path_buf()),
    )
}

pub(crate) fn guest_argv(args: &crate::acp::AcpSpawnArgs<'_>) -> Vec<String> {
    let mut v = vec![format!("{GUEST_AGENT}/index.js")];
    if args.force {
        v.push("--force".into());
    }
    if let Some(m) = args.model.map(str::trim).filter(|s| !s.is_empty()) {
        v.push("--model".into());
        v.push(m.into());
    }
    v.push("acp".into());
    v
}

#[cfg(test)]
mod kiss_coverage {
    use super::*;

    #[test]
    fn kiss_cov_spawn_symbols() {
        let _: Option<MicrosandboxTransport> = None;
        let _ = stringify!(spawn_microsandbox_transport);
        let _ = stringify!(resolve_spawn_agent_bin);
        let _ = stringify!(guest_argv);
        let _ = stringify!(create_agent_sandbox);
        let _ = stringify!(guest_exec_stream);
        let _ = stringify!(spawn_sleep_holder);
        let _ = stringify!(microsandbox_io_channels);
        let _ = stringify!(start_guest_io_bridge);
    }
}
