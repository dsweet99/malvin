use std::sync::atomic::{AtomicU64, Ordering};

use super::session::NpmPiSession;
use crate::acp::AgentError;
use crate::bridge_sdk::JsonLineSession;

static SEQ: AtomicU64 = AtomicU64::new(1);

pub(crate) fn next_id() -> String {
    format!("npm-pi-{}", SEQ.fetch_add(1, Ordering::Relaxed))
}

pub(crate) async fn npm_pi_send_prompt(
    session: &NpmPiSession,
    prompt: &str,
) -> Result<(), AgentError> {
    let id = next_id();
    session
        .write_json(&serde_json::json!({
            "id": id,
            "type": "prompt",
            "message": prompt,
        }))
        .await?;
    super::session_turn::consume_npm_pi_turn(session, &id).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_id_increments() {
        let a = next_id();
        let b = next_id();
        assert_ne!(a, b);
        assert!(a.starts_with("npm-pi-"));
    }
}
