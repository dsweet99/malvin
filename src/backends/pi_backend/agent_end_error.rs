use serde_json::Value;

use crate::agent_process::{AgentError, AgentFault};

pub(crate) const OUTPUT_CAP_MESSAGE: &str =
    "output cap reached while thinking; raise `context_size` for this model";

fn last_assistant(messages: Option<&Value>) -> Option<&Value> {
    messages?
        .as_array()?
        .iter()
        .rev()
        .find(|msg| msg.get("role").and_then(Value::as_str) == Some("assistant"))
}

fn block_is_visible(block: &Value) -> bool {
    match block.get("type").and_then(Value::as_str) {
        Some("text") => block
            .get("text")
            .and_then(Value::as_str)
            .is_some_and(|t| !t.trim().is_empty()),
        Some("toolCall") => true,
        _ => false,
    }
}

fn has_visible_output(assistant: &Value) -> bool {
    assistant
        .get("content")
        .and_then(Value::as_array)
        .is_some_and(|blocks| blocks.iter().any(block_is_visible))
}

fn stop_reason_error(assistant: &Value) -> Option<String> {
    (assistant.get("stopReason").and_then(Value::as_str) == Some("error")).then(|| {
        let detail = assistant
            .get("errorMessage")
            .and_then(Value::as_str)
            .map_or("no error message", str::trim);
        format!("pi turn failed: {detail}")
    })
}

#[must_use]
pub(super) fn agent_end_error(messages: Option<&Value>, cap: Option<u64>) -> Option<AgentError> {
    let assistant = last_assistant(messages)?;
    if let Some(message) = stop_reason_error(assistant) {
        return Some(AgentError(message));
    }
    output_cap_error(assistant, cap).map(|message| AgentError {
        message,
        fault: AgentFault::OutputCap,
    })
}

fn output_cap_error(assistant: &Value, cap: Option<u64>) -> Option<String> {
    let cap = cap?;
    let output = assistant
        .pointer("/usage/output")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let stopped_at_length = assistant.get("stopReason").and_then(Value::as_str) == Some("length");
    let capped = output >= cap || stopped_at_length;
    (capped && !has_visible_output(assistant))
        .then(|| format!("{OUTPUT_CAP_MESSAGE} (output tokens {output}, cap {cap})"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn assistant(content: &Value, output: u64, stop: &str) -> Value {
        json!([
            {"role": "user", "content": [{"type": "text", "text": "hi"}]},
            {"role": "assistant", "content": content, "usage": {"output": output}, "stopReason": stop}
        ])
    }

    #[test]
    fn thinking_only_reply_at_cap_is_an_output_cap_fault() {
        let msgs = assistant(&json!([{"type": "thinking", "thinking": "hmm"}]), 2048, "length");
        let err = agent_end_error(Some(&msgs), Some(2048)).expect("capped");
        assert_eq!(err.fault, AgentFault::OutputCap);
        assert!(
            err.message.starts_with(OUTPUT_CAP_MESSAGE) && err.message.contains("cap 2048"),
            "{}",
            err.message
        );
    }

    #[test]
    fn visible_text_or_tool_calls_are_not_errors() {
        let text = assistant(&json!([{"type": "text", "text": "done"}]), 4096, "length");
        assert!(agent_end_error(Some(&text), Some(2048)).is_none());
        let tool = assistant(&json!([{"type": "toolCall", "name": "bash"}]), 4096, "length");
        assert!(agent_end_error(Some(&tool), Some(2048)).is_none());
    }

    #[test]
    fn no_cap_or_short_reply_is_not_an_error() {
        let empty = assistant(&json!([]), 10, "stop");
        assert!(agent_end_error(Some(&empty), Some(2048)).is_none());
        assert!(agent_end_error(Some(&empty), None).is_none());
        assert!(agent_end_error(None, Some(1)).is_none());
    }

    #[test]
    fn provider_error_stop_reason_fails_the_turn() {
        let mut msgs = assistant(&json!([]), 0, "error");
        msgs[1]["errorMessage"] = json!("You have no credits remaining.");
        let err = agent_end_error(Some(&msgs), None).expect("provider error");
        assert_eq!(err.fault, AgentFault::Ordinary);
        assert_eq!(err.message, "pi turn failed: You have no credits remaining.");
    }
}
