mod auth;
mod discover;
mod extension_ui;
mod map_event;
mod models_list;
mod session;
mod session_io;
mod session_process;
mod session_spawn;
mod session_turn;

#[cfg(test)]
mod kiss_coverage_tests;
#[cfg(test)]
mod map_event_tests;

pub(crate) use auth::ensure_npm_pi_authenticated;
pub use models_list::{list_npm_pi_display_models, refresh_npm_pi_models};
pub(crate) use session::NpmPiSession;
pub(crate) use session_spawn::npm_pi_spawn_bridge as spawn_bridge;
