//! Pi JSONL RPC request/response helpers (not [`crate::bridge_protocol::BridgeRequest`]).

use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PiRequest {
    pub id: String,
    pub type_name: &'static str,
    pub extra: Value,
}

#[must_use]
pub(crate) fn pi_encode_request(req: &PiRequest) -> String {
    let mut obj = serde_json::Map::new();
    obj.insert("id".into(), Value::String(req.id.clone()));
    obj.insert("type".into(), Value::String(req.type_name.into()));
    if let Value::Object(extra) = &req.extra {
        for (k, v) in extra {
            obj.insert(k.clone(), v.clone());
        }
    }
    Value::Object(obj).to_string()
}

#[must_use]
pub(crate) fn prompt_request(id: impl Into<String>, message: &str) -> PiRequest {
    PiRequest {
        id: id.into(),
        type_name: "prompt",
        extra: json!({ "message": message }),
    }
}

#[must_use]
pub(crate) fn new_session_request(id: impl Into<String>) -> PiRequest {
    PiRequest {
        id: id.into(),
        type_name: "new_session",
        extra: json!({}),
    }
}

#[must_use]
pub(crate) fn abort_request(id: impl Into<String>) -> PiRequest {
    PiRequest {
        id: id.into(),
        type_name: "abort",
        extra: json!({}),
    }
}

/// Classify a stdout JSON line from `pi --rpc`.
#[derive(Debug, Clone)]
pub(crate) enum PiLine {
    Response {
        id: String,
        success: bool,
        error: Option<String>,
        /// Present on successful/failed command replies; kept for diagnostics/tests.
        #[allow(dead_code)]
        command: Option<String>,
        /// Optional payload from Pi; unused by the v1 adapter path.
        #[allow(dead_code)]
        data: Value,
    },
    Event {
        type_name: String,
        raw: Value,
    },
}

pub(crate) fn pi_decode_line(line: &str) -> Result<PiLine, String> {
    let value: Value =
        serde_json::from_str(line).map_err(|e| format!("pi rpc parse: {e}"))?;
    let type_name = value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    if type_name == "response" {
        return Ok(decode_response_line(&value));
    }
    Ok(PiLine::Event {
        type_name,
        raw: value,
    })
}

fn decode_response_line(value: &Value) -> PiLine {
    let id = value
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let success = value
        .get("success")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let error = json_error_string(value.get("error"));
    let command = value
        .get("command")
        .and_then(Value::as_str)
        .map(str::to_string);
    let data = value.get("data").cloned().unwrap_or(Value::Null);
    PiLine::Response {
        id,
        success,
        error,
        command,
        data,
    }
}

fn json_error_string(error: Option<&Value>) -> Option<String> {
    let e = error?;
    if e.is_null() {
        return None;
    }
    e.as_str()
        .map(str::to_string)
        .or_else(|| Some(e.to_string()))
}
