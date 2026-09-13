use serde_json::Value;

use super::session::NpmPiSession;
use crate::acp::AgentError;

pub(super) async fn auto_reply_extension_ui(
    session: &NpmPiSession,
    request: &Value,
) -> Result<(), AgentError> {
    let Some(id) = request.get("id").and_then(Value::as_str) else {
        return Ok(());
    };
    let method = request.get("method").and_then(Value::as_str).unwrap_or("");
    if is_fire_and_forget(method) {
        return Ok(());
    }
    let response = extension_ui_response(id, method, request);
    super::session_io::write_json(session, &response).await
}

fn is_fire_and_forget(method: &str) -> bool {
    matches!(
        method,
        "notify" | "setStatus" | "setWidget" | "setTitle" | "set_editor_text"
    )
}

fn extension_ui_response(id: &str, method: &str, request: &Value) -> Value {
    match method {
        "confirm" => serde_json::json!({
            "type": "extension_ui_response",
            "id": id,
            "confirmed": true
        }),
        "select" => {
            let first = request
                .get("options")
                .and_then(Value::as_array)
                .and_then(|a| a.first())
                .cloned()
                .unwrap_or(Value::Null);
            serde_json::json!({
                "type": "extension_ui_response",
                "id": id,
                "value": first
            })
        }
        _ => serde_json::json!({
            "type": "extension_ui_response",
            "id": id,
            "cancelled": true
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confirm_and_cancel_shapes() {
        let confirm = extension_ui_response("1", "confirm", &Value::Null);
        assert_eq!(confirm["confirmed"], true);
        let cancel = extension_ui_response("2", "input", &Value::Null);
        assert_eq!(cancel["cancelled"], true);
        assert!(is_fire_and_forget("notify"));
        assert!(!is_fire_and_forget("confirm"));
    }
}
