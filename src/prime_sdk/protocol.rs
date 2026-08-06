//! JSONL bridge protocol types (Prime SDK; no `resume` in v1).

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum PrimeBridgeRequest {
    Create {
        cwd: String,
        model: String,
        #[serde(rename = "apiKey", skip_serializing_if = "Option::is_none")]
        api_key: Option<String>,
        #[serde(rename = "noForcePolicy", skip_serializing_if = "Option::is_none")]
        no_force_policy: Option<&'static str>,
    },
    Send {
        prompt: String,
    },
    #[serde(rename = "cancel")]
    Cancel {},
    #[serde(rename = "close")]
    Close {},
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum PrimeBridgeEvent {
    Ok {
        #[serde(rename = "agentId")]
        agent_id: Option<String>,
    },
    Assistant { text: String },
    Thinking { text: String },
    ToolCall {
        phase: String,
        name: Option<String>,
        summary: Option<String>,
        #[serde(rename = "toolCallId")]
        tool_call_id: Option<String>,
    },
    Step { kind: Option<String> },
    Usage { usage: Value },
    RunDone {
        status: String,
        result: Option<String>,
        usage: Option<Value>,
        error: Option<String>,
        #[serde(rename = "durationMs")]
        duration_ms: Option<u64>,
    },
    Fatal {
        message: String,
        retryable: Option<bool>,
    },
    #[serde(other)]
    Unknown,
}

pub fn prime_encode_request(req: &PrimeBridgeRequest) -> Result<String, String> {
    serde_json::to_string(req).map_err(|e| e.to_string())
}

pub fn prime_decode_event(line: &str) -> Result<PrimeBridgeEvent, String> {
    serde_json::from_str(line).map_err(|e| format!("bridge event parse: {e}"))
}
