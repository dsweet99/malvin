//! JSONL bridge protocol types.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum BridgeRequest {
    Create {
        cwd: String,
        model: String,
        #[serde(rename = "apiKey", skip_serializing_if = "Option::is_none")]
        api_key: Option<String>,
        #[serde(rename = "noForcePolicy", skip_serializing_if = "Option::is_none")]
        no_force_policy: Option<&'static str>,
    },
    Resume {
        #[serde(rename = "agentId")]
        agent_id: String,
        cwd: String,
        model: String,
        #[serde(rename = "apiKey", skip_serializing_if = "Option::is_none")]
        api_key: Option<String>,
        #[serde(rename = "noForcePolicy", skip_serializing_if = "Option::is_none")]
        no_force_policy: Option<&'static str>,
    },
    Send {
        prompt: String,
        #[serde(rename = "forceStuck", skip_serializing_if = "Option::is_none")]
        force_stuck: Option<bool>,
    },
    #[serde(rename = "cancel")]
    Cancel {},
    #[serde(rename = "close")]
    Close {},
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum BridgeEvent {
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

pub fn encode_request(req: &BridgeRequest) -> Result<String, String> {
    serde_json::to_string(req).map_err(|e| e.to_string())
}

pub fn decode_event(line: &str) -> Result<BridgeEvent, String> {
    serde_json::from_str(line).map_err(|e| format!("bridge event parse: {e}"))
}
