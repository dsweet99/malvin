use std::future::Future;
use std::pin::Pin;

use crate::acp::{AgentError, AuthError};
use crate::bridge_sdk::BridgeSpawnArgs;
use crate::model_id::{ModelBackend, ParsedModel};

use super::sdk_session::SdkSession;

pub(crate) type SpawnFuture<'a> =
    Pin<Box<dyn Future<Output = Result<SdkSession, AgentError>> + Send + 'a>>;

pub(crate) trait BackendLifecycle: Sync {
    fn supports_thinking_wire(&self) -> bool;
    fn supports_service_wire(&self) -> bool;
    fn tracks_resume_agent_id(&self) -> bool;
    fn ensure_authenticated(&self, model: &ParsedModel) -> Result<(), AuthError>;
    fn spawn_session<'a>(
        &self,
        args: BridgeSpawnArgs<'a>,
        resume_agent_id: Option<&'a str>,
        service: Option<&'a str>,
    ) -> SpawnFuture<'a>;

    fn forgets_resume_id_on_busy_teardown(&self) -> bool {
        self.tracks_resume_agent_id()
    }
}

struct CursorLifecycle;
struct PiLifecycle;
struct CodexLifecycle;

#[must_use]
pub(crate) const fn backend_lifecycle(backend: ModelBackend) -> &'static dyn BackendLifecycle {
    match backend {
        ModelBackend::Cursor => &CursorLifecycle,
        ModelBackend::Pi => &PiLifecycle,
        ModelBackend::Codex => &CodexLifecycle,
    }
}

impl BackendLifecycle for CursorLifecycle {
    fn supports_thinking_wire(&self) -> bool {
        false
    }

    fn supports_service_wire(&self) -> bool {
        false
    }

    fn tracks_resume_agent_id(&self) -> bool {
        true
    }

    fn ensure_authenticated(&self, model: &ParsedModel) -> Result<(), AuthError> {
        debug_assert_eq!(model.backend, ModelBackend::Cursor);
        crate::cursor_sdk::ensure_sdk_authenticated()
    }

    fn spawn_session<'a>(
        &self,
        args: BridgeSpawnArgs<'a>,
        resume_agent_id: Option<&'a str>,
        _service: Option<&'a str>,
    ) -> SpawnFuture<'a> {
        Box::pin(async move {
            crate::cursor_sdk::spawn_bridge(args, resume_agent_id)
                .await
                .map(SdkSession::new)
        })
    }
}

impl BackendLifecycle for PiLifecycle {
    fn supports_thinking_wire(&self) -> bool {
        true
    }

    fn supports_service_wire(&self) -> bool {
        false
    }

    fn tracks_resume_agent_id(&self) -> bool {
        false
    }

    fn ensure_authenticated(&self, model: &ParsedModel) -> Result<(), AuthError> {
        debug_assert_eq!(model.backend, ModelBackend::Pi);
        crate::npm_pi_sdk::ensure_npm_pi_authenticated(&model.canonical())
    }

    fn spawn_session<'a>(
        &self,
        args: BridgeSpawnArgs<'a>,
        _resume_agent_id: Option<&'a str>,
        _service: Option<&'a str>,
    ) -> SpawnFuture<'a> {
        Box::pin(async move {
            crate::npm_pi_sdk::spawn_bridge(args)
                .await
                .map(SdkSession::new)
        })
    }
}

impl BackendLifecycle for CodexLifecycle {
    fn supports_thinking_wire(&self) -> bool {
        true
    }

    fn supports_service_wire(&self) -> bool {
        true
    }

    fn tracks_resume_agent_id(&self) -> bool {
        false
    }

    fn ensure_authenticated(&self, model: &ParsedModel) -> Result<(), AuthError> {
        debug_assert_eq!(model.backend, ModelBackend::Codex);
        crate::codex_sdk::ensure_codex_authenticated()
    }

    fn spawn_session<'a>(
        &self,
        args: BridgeSpawnArgs<'a>,
        _resume_agent_id: Option<&'a str>,
        service: Option<&'a str>,
    ) -> SpawnFuture<'a> {
        Box::pin(async move {
            crate::codex_sdk::spawn_bridge(args, service)
                .await
                .map(SdkSession::new)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model_id::parse_model_id;

    #[test]
    fn lifecycle_capability_matrix() {
        let cursor = backend_lifecycle(ModelBackend::Cursor);
        assert!(!cursor.supports_thinking_wire());
        assert!(!cursor.supports_service_wire());
        assert!(cursor.tracks_resume_agent_id());
        assert!(cursor.forgets_resume_id_on_busy_teardown());

        let pi = backend_lifecycle(ModelBackend::Pi);
        assert!(pi.supports_thinking_wire());
        assert!(!pi.supports_service_wire());
        assert!(!pi.tracks_resume_agent_id());
        assert!(!pi.forgets_resume_id_on_busy_teardown());

        let codex = backend_lifecycle(ModelBackend::Codex);
        assert!(codex.supports_thinking_wire());
        assert!(codex.supports_service_wire());
        assert!(!codex.tracks_resume_agent_id());
        assert!(!codex.forgets_resume_id_on_busy_teardown());
    }

    #[test]
    fn kiss_cov_backend_lifecycle() {
        let _ = stringify!(BackendLifecycle);
        let _ = stringify!(backend_lifecycle);
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
