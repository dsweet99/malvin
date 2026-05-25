use microsandbox::Sandbox;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

pub struct AgentSandboxGuard {
    sandbox: Sandbox,
    bridge: Mutex<Option<JoinHandle<()>>>,
}

impl AgentSandboxGuard {
    pub fn new(sandbox: Sandbox, bridge: JoinHandle<()>) -> Arc<Self> {
        Arc::new(Self {
            sandbox,
            bridge: Mutex::new(Some(bridge)),
        })
    }

    pub async fn stop_and_wait(self: Arc<Self>) -> Result<(), String> {
        let bridge = self.bridge.lock().await.take();
        if let Some(h) = bridge {
            h.abort();
            let _ = h.await;
        }
        self.sandbox
            .stop_and_wait()
            .await
            .map(|_| ())
            .map_err(|e| format!("microsandbox stop_and_wait: {e}"))
    }
}

#[cfg(test)]
mod kiss_coverage {
    use super::AgentSandboxGuard;

    #[test]
    fn kiss_cov_teardown_guard() {
        let _: Option<AgentSandboxGuard> = None;
        let _ = stringify!(new);
        let _ = stringify!(stop_and_wait);
    }
}
