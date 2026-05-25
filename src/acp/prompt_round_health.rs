//! Per-prompt summary of ACP tool failures observed during a live `session/prompt`.

use serde_json::Value;

const MAX_TOOL_ERRORS: usize = 4;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PromptRoundHealth {
    tool_errors: Vec<String>,
    silent_shell_completions: u32,
    agent_streamed_kpop_solved: bool,
}

impl PromptRoundHealth {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn record_session_update(&mut self, msg: &Value) {
        let Some(update) = msg
            .get("params")
            .and_then(|p| p.get("update"))
            .and_then(|u| u.as_object())
        else {
            return;
        };
        if update.get("sessionUpdate").and_then(|v| v.as_str()) == Some("agent_message_chunk") {
            self.record_agent_chunk(update);
            return;
        }
        self.record_completed_tool_call(update);
    }

    fn record_silent_shell_completion(&mut self, update: &serde_json::Map<String, Value>, raw: &serde_json::Map<String, Value>) {
        if update.get("kind").and_then(|v| v.as_str()) == Some("execute")
            && raw.get("exitCode").and_then(serde_json::Value::as_u64) == Some(0)
            && raw_output_text_empty(raw)
        {
            self.silent_shell_completions = self.silent_shell_completions.saturating_add(1);
        }
    }

    fn record_completed_tool_call(&mut self, update: &serde_json::Map<String, Value>) {
        if update.get("sessionUpdate").and_then(|v| v.as_str()) != Some("tool_call_update") {
            return;
        }
        if update.get("status").and_then(|v| v.as_str()) != Some("completed") {
            return;
        }
        let Some(raw) = update.get("rawOutput").and_then(|v| v.as_object()) else {
            return;
        };
        if let Some(err) = raw.get("error").and_then(|v| v.as_str()) {
            self.record_tool_error(err, update);
            return;
        }
        self.record_silent_shell_completion(update, raw);
    }

    fn record_agent_chunk(&mut self, update: &serde_json::Map<String, Value>) {
        let text = update
            .get("content")
            .and_then(|c| c.get("text"))
            .and_then(|t| t.as_str())
            .or_else(|| update.get("content").and_then(|c| c.as_str()));
        let Some(text) = text else {
            return;
        };
        if text.contains("## KPOP_SOLVED") {
            self.agent_streamed_kpop_solved = true;
        }
    }

    fn record_tool_error(&mut self, err: &str, update: &serde_json::Map<String, Value>) {
        if self.tool_errors.len() >= MAX_TOOL_ERRORS {
            return;
        }
        let kind = update
            .get("kind")
            .and_then(|v| v.as_str())
            .unwrap_or("tool");
        let title = update
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let detail = if title.is_empty() {
            format!("{kind}: {err}")
        } else {
            format!("{kind} ({title}): {err}")
        };
        if !self.tool_errors.iter().any(|e| e == &detail) {
            self.tool_errors.push(detail);
        }
    }

    #[must_use]
    pub const fn has_infra_failure(&self) -> bool {
        !self.tool_errors.is_empty() || self.silent_shell_completions >= 2
    }

    #[must_use]
    pub const fn agent_streamed_kpop_solved(&self) -> bool {
        self.agent_streamed_kpop_solved
    }

    #[must_use]
    pub fn format_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        for err in &self.tool_errors {
            lines.push(format!("  - {err}"));
        }
        if self.silent_shell_completions > 0 {
            lines.push(format!(
                "  - execute: exit 0 with empty stdout/stderr (×{})",
                self.silent_shell_completions
            ));
        }
        if self.agent_streamed_kpop_solved {
            lines.push(
                "  - agent streamed `## KPOP_SOLVED` in chat during this prompt".to_string(),
            );
        }
        lines
    }
}

fn raw_output_text_empty(raw: &serde_json::Map<String, Value>) -> bool {
    let stdout = raw.get("stdout").and_then(|v| v.as_str()).unwrap_or("");
    let stderr = raw.get("stderr").and_then(|v| v.as_str()).unwrap_or("");
    stdout.is_empty() && stderr.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn records_service_unavailable_on_search_tool() {
        let msg = json!({
            "method": "session/update",
            "params": {"update": {
                "sessionUpdate": "tool_call_update",
                "kind": "search",
                "title": "Find",
                "status": "completed",
                "rawOutput": {"error": "Service temporarily unavailable. This may be temporary; try again."}
            }}
        });
        let mut h = PromptRoundHealth::default();
        h.record_session_update(&msg);
        assert!(h.has_infra_failure());
        assert!(h.format_lines()[0].contains("Service temporarily unavailable"));
    }

    #[test]
    fn counts_silent_shell_completions() {
        let msg = json!({
            "method": "session/update",
            "params": {"update": {
                "sessionUpdate": "tool_call_update",
                "kind": "execute",
                "status": "completed",
                "rawOutput": {"exitCode": 0, "stdout": "", "stderr": ""}
            }}
        });
        let mut h = PromptRoundHealth::default();
        h.record_session_update(&msg);
        h.record_session_update(&msg);
        assert!(h.has_infra_failure());
    }

    #[test]
    fn detects_streamed_kpop_solved_in_agent_chunk() {
        let msg = json!({
            "method": "session/update",
            "params": {"update": {
                "sessionUpdate": "agent_message_chunk",
                "content": {"text": "## KPOP_SOLVED\nDone.", "type": "text"}
            }}
        });
        let mut h = PromptRoundHealth::default();
        h.record_session_update(&msg);
        assert!(h.agent_streamed_kpop_solved());
    }
}
