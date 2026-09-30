use crate::acp::{AgentError, AuthError};
use crate::bridge_sdk::BridgeSpawnArgs;
use crate::model_id::{ModelBackend, ParsedModel};

use super::sdk_session::SdkSession;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BackendLifecycle(ModelBackend);

impl BackendLifecycle {
    #[must_use]
    pub const fn of(backend: ModelBackend) -> Self {
        Self(backend)
    }

    #[must_use]
    pub const fn supports_thinking_wire(self) -> bool {
        matches!(
            self.0,
            ModelBackend::NpmPi | ModelBackend::Pi | ModelBackend::Codex
        )
    }

    #[must_use]
    pub const fn supports_service_wire(self) -> bool {
        matches!(self.0, ModelBackend::Codex)
    }

    #[must_use]
    pub const fn tracks_resume_agent_id(self) -> bool {
        matches!(self.0, ModelBackend::Cursor)
    }

    #[must_use]
    pub const fn forgets_resume_id_on_busy_teardown(self) -> bool {
        self.tracks_resume_agent_id()
    }

    pub fn ensure_authenticated(self, model: &ParsedModel) -> Result<(), AuthError> {
        debug_assert_eq!(self.0, model.backend);
        match self.0 {
            ModelBackend::Cursor => crate::cursor_sdk::ensure_sdk_authenticated(),
            ModelBackend::NpmPi => {
                crate::npm_pi_sdk::ensure_npm_pi_authenticated(&model.canonical())
            }
            ModelBackend::Pi => crate::pi_sdk::ensure_pi_authenticated(&model.canonical()),
            ModelBackend::Codex => crate::codex_sdk::ensure_codex_authenticated(),
        }
    }

    pub async fn spawn_session(
        self,
        args: BridgeSpawnArgs<'_>,
        resume_agent_id: Option<&str>,
        service: Option<&str>,
    ) -> Result<SdkSession, AgentError> {
        match self.0 {
            ModelBackend::Cursor => crate::cursor_sdk::spawn_bridge(args, resume_agent_id)
                .await
                .map(|session| SdkSession::Cursor(Box::new(session))),
            ModelBackend::NpmPi => crate::npm_pi_sdk::spawn_bridge(args)
                .await
                .map(|session| SdkSession::NpmPi(Box::new(session))),
            ModelBackend::Pi => crate::pi_sdk::spawn_bridge(args).await,
            ModelBackend::Codex => crate::codex_sdk::spawn_bridge(args, service)
                .await
                .map(|session| SdkSession::Codex(Box::new(session))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model_id::parse_model_id;

    #[test]
    fn lifecycle_capability_matrix() {
        let cursor = BackendLifecycle::of(ModelBackend::Cursor);
        assert!(!cursor.supports_thinking_wire());
        assert!(!cursor.supports_service_wire());
        assert!(cursor.tracks_resume_agent_id());
        assert!(cursor.forgets_resume_id_on_busy_teardown());

        let npm = BackendLifecycle::of(ModelBackend::NpmPi);
        assert!(npm.supports_thinking_wire());
        assert!(!npm.supports_service_wire());
        assert!(!npm.tracks_resume_agent_id());

        let pi = BackendLifecycle::of(ModelBackend::Pi);
        assert!(pi.supports_thinking_wire());
        assert!(!pi.supports_service_wire());
        assert!(!pi.tracks_resume_agent_id());

        let codex = BackendLifecycle::of(ModelBackend::Codex);
        assert!(codex.supports_thinking_wire());
        assert!(codex.supports_service_wire());
        assert!(!codex.tracks_resume_agent_id());
    }

    #[test]
    fn lifecycle_of_round_trips_backend() {
        for b in [
            ModelBackend::Cursor,
            ModelBackend::NpmPi,
            ModelBackend::Pi,
            ModelBackend::Codex,
        ] {
            let life = BackendLifecycle::of(b);
            assert_eq!(life.0, b);
        }
        let _ = stringify!(BackendLifecycle);
        let _ = stringify!(supports_thinking_wire);
        let _ = stringify!(supports_service_wire);
        let _ = stringify!(tracks_resume_agent_id);
        let _ = stringify!(forgets_resume_id_on_busy_teardown);
        let _ = stringify!(ensure_authenticated);
        let _ = stringify!(spawn_session);
        let model = parse_model_id("cursor:auto").expect("model");
        assert_eq!(model.backend.bridge_wire_model(&model), "auto");
    }
}
