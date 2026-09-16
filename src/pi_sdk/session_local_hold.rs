use super::PiEmbeddedSession;

impl PiEmbeddedSession {
    pub(crate) fn release_local_hold(&mut self) {
        if !self.local_hold {
            return;
        }
        self.local_hold = false;
        if let Err(e) = crate::pi_sdk::local_lifecycle::release_local_llm() {
            tracing::warn!(target: "malvin::pi_sdk", error = %e, "local llm release failed");
        }
    }
}

impl Drop for PiEmbeddedSession {
    fn drop(&mut self) {
        self.release_local_hold();
        self.reader_dead
            .store(true, std::sync::atomic::Ordering::SeqCst);
        if let Some(mut runtime) = self.runtime.take() {
            runtime.abort();
            let _ = runtime.shutdown();
        }
        #[cfg(unix)]
        {
            crate::acp::terminate_agent_process_group_for_interrupt(None, &self.spawn_pid_baseline);
        }
        crate::malvin_sandbox::clear_active_sandbox_session();
    }
}
