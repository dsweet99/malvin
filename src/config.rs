//! Runtime configuration shared with the vendored ACP client.

/// Default for `malvin` `--model` when the flag is omitted.
pub const DEFAULT_CLI_MODEL: &str = "auto";

/// Default JSON-RPC wait for `agent acp` (seconds). Override with `MALVIN_ACP_RPC_TIMEOUT_SECS`.
pub const DEFAULT_ACP_RPC_TIMEOUT_SECS: u64 = 600;

/// Reads `MALVIN_ACP_RPC_TIMEOUT_SECS` (seconds, minimum **1**).
#[must_use]
pub fn acp_rpc_timeout_secs_from_env() -> u64 {
    std::env::var("MALVIN_ACP_RPC_TIMEOUT_SECS")
        .ok()
        .map_or(DEFAULT_ACP_RPC_TIMEOUT_SECS, |s| {
            s.parse::<u64>().map_or_else(
                |_| {
                    tracing::warn!(
                        target: "malvin::config",
                        value = %s,
                        "MALVIN_ACP_RPC_TIMEOUT_SECS is not a positive integer; using default"
                    );
                    DEFAULT_ACP_RPC_TIMEOUT_SECS
                },
                |n| n.max(1),
            )
        })
}
