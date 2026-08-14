
use super::sdk_client::SdkClient;

pub type AgentBackend = SdkClient;

#[must_use]
pub const fn agent_backend_from_client(client: SdkClient) -> AgentBackend {
    client
}
