//! Shared JSONL bridge protocol for Cursor and Prime Node bridges.

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
        /// Temp Prime `models.json` for `prime:local/…` sidecar.
        #[serde(rename = "modelsJsonPath", skip_serializing_if = "Option::is_none")]
        models_json_path: Option<String>,
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

#[cfg(test)]
mod bridge_protocol_tests {
    use super::*;

    #[test]
    fn encode_create_optional_models_json_and_api_key() {
        let cursor = encode_request(&BridgeRequest::Create {
            cwd: "/tmp".into(),
            model: "auto".into(),
            api_key: Some("k".into()),
            no_force_policy: Some("fail_fast"),
            models_json_path: None,
        })
        .expect("encode");
        assert!(cursor.contains("\"apiKey\":\"k\""));
        assert!(cursor.contains("\"noForcePolicy\":\"fail_fast\""));
        assert!(!cursor.contains("modelsJsonPath"));

        let prime_local = encode_request(&BridgeRequest::Create {
            cwd: "/tmp".into(),
            model: "local/qwen35_9b_q4".into(),
            api_key: None,
            no_force_policy: None,
            models_json_path: Some("/tmp/models.json".into()),
        })
        .expect("local create");
        assert!(prime_local.contains("modelsJsonPath"));
        assert!(!prime_local.contains("apiKey"));
    }

    #[test]
    fn encode_send_skips_force_stuck_when_none() {
        let send = encode_request(&BridgeRequest::Send {
            prompt: "hi".into(),
            force_stuck: None,
        })
        .expect("send");
        assert!(send.contains("\"op\":\"send\""));
        assert!(!send.contains("forceStuck"));
    }

    #[test]
    fn encode_resume_uses_agent_id() {
        let line = encode_request(&BridgeRequest::Resume {
            agent_id: "bc-123".into(),
            cwd: "/tmp".into(),
            model: "auto".into(),
            api_key: Some("k".into()),
            no_force_policy: None,
        })
        .expect("encode");
        assert!(line.contains("\"op\":\"resume\""));
        assert!(line.contains("\"agentId\":\"bc-123\""));
    }

    #[test]
    fn decode_run_done_and_fatal() {
        let done = decode_event(
            r#"{"event":"run_done","status":"finished","result":"hi","usage":{"inputTokens":1,"outputTokens":2}}"#,
        )
        .expect("decode");
        match done {
            BridgeEvent::RunDone {
                status, result, ..
            } => {
                assert_eq!(status, "finished");
                assert_eq!(result.as_deref(), Some("hi"));
            }
            other => panic!("unexpected {other:?}"),
        }
        let fatal = decode_event(r#"{"event":"fatal","message":"boom","retryable":true}"#)
            .expect("fatal");
        match fatal {
            BridgeEvent::Fatal { message, retryable } => {
                assert_eq!(message, "boom");
                assert_eq!(retryable, Some(true));
            }
            other => panic!("unexpected {other:?}"),
        }
    }
}
