//! Shutdown helpers: classify rejected `session/cancel` and bound child waits.

use std::time::Duration;

#[must_use]
pub(crate) fn cancel_rejected_as_unsupported(err: &str) -> bool {
    let lower = err.to_ascii_lowercase();
    lower.contains("method not found")
        || lower.contains("not supported")
        || lower.contains("-32601")
}

/// Bound `session/cancel` before OS teardown; skip extending wait on unsupported cancel.
pub(crate) async fn best_effort_session_cancel<F>(cancel: F, cancel_timeout: Duration)
where
    F: std::future::Future<Output = Result<(), String>>,
{
    if cancel_timeout.is_zero() {
        let _ = cancel.await;
        return;
    }
    match tokio::time::timeout(cancel_timeout, cancel).await {
        Ok(Err(err)) if cancel_rejected_as_unsupported(&err) => {}
        Ok(Ok(()) | Err(_)) | Err(_) => {}
    }
}

/// Wait for a killed ACP child with a hard timeout so shutdown cannot hang.
pub(crate) async fn wait_killed_child(
    ch: &mut tokio::process::Child,
    wait_budget: Duration,
) -> Result<(), String> {
    if let Ok(r) = tokio::time::timeout(wait_budget, ch.wait()).await {
        r.map_err(|e| format!("acp wait: {e}"))?;
    } else {
        let _ = ch.kill().await;
        let _ = tokio::time::timeout(Duration::from_millis(200), ch.wait()).await;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        best_effort_session_cancel, cancel_rejected_as_unsupported, wait_killed_child,
    };
    use std::time::Duration;

    #[test]
    fn detects_method_not_found() {
        assert!(cancel_rejected_as_unsupported(
            "Method not found: session/cancel"
        ));
        assert!(cancel_rejected_as_unsupported("jsonrpc error -32601"));
        assert!(cancel_rejected_as_unsupported("operation not supported"));
        assert!(!cancel_rejected_as_unsupported("request timed out"));
    }

    #[tokio::test]
    async fn best_effort_cancel_covers_branches() {
        best_effort_session_cancel(async { Ok(()) }, Duration::ZERO).await;
        best_effort_session_cancel(
            async { Err("Method not found".into()) },
            Duration::from_millis(50),
        )
        .await;
        best_effort_session_cancel(
            async { Err("transient rpc failure".into()) },
            Duration::from_millis(50),
        )
        .await;
        best_effort_session_cancel(async { Ok(()) }, Duration::from_millis(50)).await;
        best_effort_session_cancel(
            async {
                tokio::time::sleep(Duration::from_secs(30)).await;
                Ok(())
            },
            Duration::from_millis(10),
        )
        .await;
    }

    #[tokio::test]
    async fn wait_killed_child_reaps_sleep() {
        let mut child = tokio::process::Command::new("sleep")
            .arg("30")
            .kill_on_drop(true)
            .spawn()
            .expect("spawn sleep");
        let _ = child.kill().await;
        wait_killed_child(&mut child, Duration::from_millis(500))
            .await
            .expect("wait");
    }

    #[tokio::test]
    async fn wait_killed_child_times_out_then_kills() {
        let mut child = tokio::process::Command::new("sleep")
            .arg("30")
            .kill_on_drop(true)
            .spawn()
            .expect("spawn sleep");
        // Tiny budget forces the timeout arm; helper must SIGKILL and return Ok.
        wait_killed_child(&mut child, Duration::from_millis(1))
            .await
            .expect("timeout path");
    }
}
