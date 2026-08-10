//! Unified agent backend (single [`SdkClient`] surface).

use super::sdk_client::SdkClient;

/// Product agent backend: one Cursor-or-Prime [`SdkClient`].
pub type AgentBackend = SdkClient;

/// Construct an [`AgentBackend`] from a configured [`SdkClient`].
#[must_use]
pub const fn agent_backend_from_client(client: SdkClient) -> AgentBackend {
    client
}
