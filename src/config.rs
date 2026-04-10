//! Runtime configuration shared with the vendored ACP client.

/// Default JSON-RPC wait for `agent acp` (seconds). Override with `MALVIN_ACP_RPC_TIMEOUT_SECS`.
pub const DEFAULT_ACP_RPC_TIMEOUT_SECS: u64 = 600;

/// Reads `MALVIN_ACP_RPC_TIMEOUT_SECS` (seconds, minimum **1**).
#[must_use]
pub fn acp_rpc_timeout_secs_from_env() -> u64 {
    std::env::var("MALVIN_ACP_RPC_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map_or(DEFAULT_ACP_RPC_TIMEOUT_SECS, |n| n.max(1))
}
