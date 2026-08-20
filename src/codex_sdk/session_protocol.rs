use crate::acp::AgentError;
use crate::bridge_sdk::BridgeSession;

pub(crate) async fn codex_initialize(session: &BridgeSession) -> Result<(), AgentError> {
    let response = request(
        session,
        "initialize",
        serde_json::json!({
            "clientInfo": {
                "name": "malvin",
                "title": "Malvin",
                "version": env!("CARGO_PKG_VERSION")
            },
            "capabilities": {
                "experimentalApi": true
            }
        }),
    )
    .await?;
    if response.get("error").is_some() {
        return Err(response_error("codex initialize", &response));
    }
    write(
        session,
        &serde_json::json!({"method":"initialized","params":{}}),
    )
    .await
}

pub(crate) async fn codex_start_thread(
    session: &BridgeSession,
    model: &str,
    cwd: &std::path::Path,
) -> Result<(), AgentError> {
    let model = crate::codex_sdk::resolve_codex_model(model).map_err(AgentError)?;
    let response = request(
        session,
        "thread/start",
        serde_json::json!({
            "model": model,
            "cwd": cwd,
            "approvalPolicy": "never",
            "sandbox": "workspace-write"
        }),
    )
    .await?;
    if response.get("error").is_some() {
        return Err(response_error("codex thread/start", &response));
    }
    let id = response
        .pointer("/result/thread/id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AgentError("codex thread/start response missing thread id".into()))?;
    *session
        .agent_id
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(id.to_owned());
    Ok(())
}

pub(crate) async fn request(
    session: &BridgeSession,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, AgentError> {
    let id = super::session_io::next_id();
    write(
        session,
        &serde_json::json!({"method": method, "id": id, "params": params}),
    )
    .await?;
    loop {
        let value = super::session_io::read_json_waiting(session, "rpc reply").await?;
        if value.get("id").and_then(serde_json::Value::as_u64) == Some(id) {
            return Ok(value);
        }
    }
}

pub(crate) fn response_error(context: &str, response: &serde_json::Value) -> AgentError {
    AgentError(format!(
        "{context}: {}",
        response.get("error").unwrap_or(response)
    ))
}

async fn write(session: &BridgeSession, value: &serde_json::Value) -> Result<(), AgentError> {
    super::session_io::write_json(session, value).await
}
