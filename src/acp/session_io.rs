//! Small stdio / RPC wiring helpers for [`super::session::AcpSession`].
use super::session_types::AcpSessionInner;
use std::time::Duration;
use tokio::process::{Child, ChildStdin, ChildStdout};

pub const fn clamp_rpc_timeout(d: Duration) -> Duration {
    if d.is_zero() {
        Duration::from_millis(1)
    } else {
        d
    }
}

pub fn acp_stdio(s: &AcpSessionInner) -> super::AcpStdioRpc {
    super::AcpStdioRpc {
        reader_dead: s.reader_dead.clone(),
        stdin: s.stdin.clone(),
        pending: s.pending.clone(),
        acp_verbose: s.acp_verbose,
    }
}

pub async fn take_stdio_pipes(
    child: &mut Child,
) -> Result<(ChildStdin, ChildStdout), String> {
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| "agent acp stdin pipe missing".to_string())?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "agent acp stdout pipe missing".to_string())?;
    Ok((stdin, stdout))
}

#[test]
fn kiss_stringify_session_io() {
    let _ = stringify!(clamp_rpc_timeout);
    let _ = stringify!(acp_stdio);
    let _ = stringify!(take_stdio_pipes);
}
