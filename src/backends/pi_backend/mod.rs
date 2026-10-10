pub(crate) use super::pi_backend_agent_end_error as agent_end_error;
pub(crate) use super::pi_backend_auth as auth;
pub(crate) use super::pi_backend_discover as discover;
pub(crate) use super::pi_backend_extension_ui as extension_ui;
pub(crate) use super::pi_backend_map_event as map_event;
pub(crate) use super::pi_backend_map_event_summary as map_event_summary;
pub(crate) use super::pi_backend_models_list as models_list;
pub(crate) use super::pi_backend_models_rpc as models_rpc;
pub(crate) use super::pi_backend_provider_auth as provider_auth;
pub(crate) use super::pi_backend_session as session;
pub(crate) use super::pi_backend_session_io as session_io;
pub(crate) use super::pi_backend_session_process as session_process;
pub(crate) use super::pi_backend_session_spawn as session_spawn;
pub(crate) use super::pi_backend_session_turn as session_turn;

#[cfg(test)]
mod issue43_drain_tests;
#[cfg(test)]
mod kiss_coverage_tests;
#[cfg(test)]
mod map_event_tests;
#[cfg(test)]
mod turn_timeout_tests;

pub(crate) use auth::ensure_npm_pi_authenticated;
pub use models_list::{list_npm_pi_display_models, refresh_npm_pi_models};
pub(crate) use session::NpmPiSession;
pub(crate) use session_spawn::npm_pi_spawn_bridge as spawn_bridge;

pub(crate) use map_event_summary::tool_summary_from_pi;
pub(crate) use provider_auth::missing_credentials_hint;
pub use provider_auth::{is_provider_authenticated, provider_has_known_credentials};
