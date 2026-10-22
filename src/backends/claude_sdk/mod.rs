pub(crate) use super::claude_sdk_auth as auth;
pub use super::claude_sdk_bridge_install as bridge_install;
pub use super::claude_sdk_bridge_path as bridge_path;
pub(crate) use super::claude_sdk_session_spawn as session_spawn;

pub use auth::ensure_claude_authenticated;
pub(crate) use session_spawn::claude_spawn_bridge as spawn_bridge;

#[cfg(test)]
#[path = "auth_tests.rs"]
mod auth_tests;
