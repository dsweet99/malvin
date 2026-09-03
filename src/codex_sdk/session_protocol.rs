use super::session::CodexSession;
use crate::acp::AgentError;

pub(crate) async fn codex_initialize(session: &CodexSession) -> Result<(), AgentError> {
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
    session: &CodexSession,
    model: &str,
    cwd: &std::path::Path,
) -> Result<(), AgentError> {
    let response = request(
        session,
        "thread/start",
        resolved_thread_start_params(model, cwd, session.service.as_deref())?,
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
        .thread_id
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(id.to_owned());
    Ok(())
}

pub(crate) async fn request(
    session: &CodexSession,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, AgentError> {
    let id = super::session_io::next_id();
    write(
        session,
        &serde_json::json!({"method": method, "id": id, "params": params}),
    )
    .await?;
    let mut turn = crate::bridge_sdk::DrainIdleTurn::new();
    loop {
        turn.check_max_deadline(crate::bridge_sdk::DrainIdleLabels {
            prefix: crate::acp::DRAIN_IDLE_PREFIX_CODEX,
            waiting_for: "rpc reply",
        })?;
        let value = super::session_io::read_json_waiting(session, "rpc reply", &mut turn).await?;
        if value.get("id").and_then(serde_json::Value::as_u64) == Some(id) {
            return Ok(value);
        }
        turn.check_max_deadline(crate::bridge_sdk::DrainIdleLabels {
            prefix: crate::acp::DRAIN_IDLE_PREFIX_CODEX,
            waiting_for: "rpc reply",
        })?;
    }
}

fn resolved_thread_start_params(
    model: &str,
    cwd: &std::path::Path,
    service: Option<&str>,
) -> Result<serde_json::Value, AgentError> {
    let model = super::discover::resolve_codex_model(model).map_err(AgentError)?;
    Ok(thread_start_params(
        model,
        cwd,
        "danger-full-access",
        service,
    ))
}

fn thread_start_params(
    model: String,
    cwd: &std::path::Path,
    sandbox: &str,
    service: Option<&str>,
) -> serde_json::Value {
    let mut params = serde_json::json!({
        "model": model,
        "cwd": cwd,
        "approvalPolicy": "never",
        "sandbox": sandbox,
        "ephemeral": true
    });
    if let Some(service) = service {
        params["serviceTier"] = serde_json::Value::String(service.to_owned());
    }
    params
}

pub(crate) fn response_error(context: &str, response: &serde_json::Value) -> AgentError {
    AgentError(format!(
        "{context}: {}",
        response.get("error").unwrap_or(response)
    ))
}

async fn write(session: &CodexSession, value: &serde_json::Value) -> Result<(), AgentError> {
    super::session_io::write_json(session, value).await
}

#[cfg(test)]
mod tests {
    use super::thread_start_params;
    use std::path::Path;

    #[test]
    fn thread_start_includes_optional_service() {
        let with_service = thread_start_params(
            "gpt-5.6-sol".into(),
            Path::new("/work"),
            "danger-full-access",
            Some("priority"),
        );
        assert_eq!(with_service["serviceTier"], "priority");
        assert_eq!(with_service["model"], "gpt-5.6-sol");
        assert_eq!(with_service["sandbox"], "danger-full-access");
        let bare = thread_start_params(
            "gpt-5.6-sol".into(),
            Path::new("/work"),
            "danger-full-access",
            None,
        );
        assert!(bare.get("serviceTier").is_none());
        assert_eq!(bare["sandbox"], "danger-full-access");
    }
}
