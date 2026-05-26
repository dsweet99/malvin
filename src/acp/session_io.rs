use std::time::Duration;

use tokio::process::{Child, ChildStdin, ChildStdout};

use super::super::session_types::AcpSessionInner;

pub const fn clamp_rpc_timeout(d: Duration) -> Duration {
    if d.is_zero() {
        Duration::from_millis(1)
    } else {
        d
    }
}

pub fn acp_stdio(s: &AcpSessionInner) -> crate::acp::AcpStdioRpc {
    crate::acp::AcpStdioRpc {
        reader_dead: s.reader_dead.clone(),
        stdin: s.stdin.clone(),
        pending: s.pending.clone(),
        acp_activity_seq: s.acp_activity_seq.clone(),
        acp_activity_notify: s.acp_activity_notify.clone(),
        acp_verbose: s.acp_verbose,
        trace_jsonl: s.trace_jsonl.clone(),
    }
}

pub async fn take_stdio_pipes(child: &mut Child) -> Result<(ChildStdin, ChildStdout), String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_rpc_timeout_zero_normalized_to_one_ms() {
        assert_eq!(clamp_rpc_timeout(Duration::ZERO), Duration::from_millis(1));
    }

    #[test]
    fn clamp_rpc_timeout_nonzero_unchanged() {
        let d = Duration::from_millis(500);
        assert_eq!(clamp_rpc_timeout(d), d);
    }

    #[tokio::test]
    async fn take_stdio_pipes_from_piped_spawn() {
        let mut child = tokio::process::Command::new("cat")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("spawn cat");
        let (stdin, stdout) = take_stdio_pipes(&mut child).await.expect("take pipes");
        child.kill().await.ok();
        drop(stdin);
        drop(stdout);
        let _ = child.wait().await;
    }
}
