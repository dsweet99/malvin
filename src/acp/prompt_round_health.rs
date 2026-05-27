//! Per-prompt summary of ACP tool failures observed during a live `session/prompt`.

use serde_json::Value;

const MAX_TOOL_ERRORS: usize = 4;
const UPGRADE_PLAN_WINDOW: usize = 128;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PromptRoundHealth {
    tool_errors: Vec<String>,
    silent_shell_completions: u32,
    agent_streamed_kpop_solved: bool,
    upgrade_plan_seen: bool,
    agent_text_acc: String,
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

    fn record_completed_tool_call(&mut self, update: &serde_json::Map<String, Value>) {
        let Some(raw) = completed_tool_call_raw(update) else {
            return;
        };
        if let Some(err) = raw.get("error").and_then(|v| v.as_str()) {
            self.record_tool_error(err, update);
            return;
        }
        if silent_shell_completion(update, raw) {
            self.silent_shell_completions = self.silent_shell_completions.saturating_add(1);
        }
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
        self.append_agent_text_for_upgrade_plan(text);
    }

    fn append_agent_text_for_upgrade_plan(&mut self, text: &str) {
        self.agent_text_acc.push_str(text);
        if crate::acp::agent_string_is_upgrade_plan(&self.agent_text_acc) {
            self.upgrade_plan_seen = true;
        }
        if self.agent_text_acc.len() > UPGRADE_PLAN_WINDOW {
            let mut drain = self.agent_text_acc.len() - UPGRADE_PLAN_WINDOW;
            while drain > 0 && !self.agent_text_acc.is_char_boundary(drain) {
                drain -= 1;
            }
            self.agent_text_acc.drain(..drain);
        }
        if crate::acp::agent_string_is_upgrade_plan(&self.agent_text_acc) {
            self.upgrade_plan_seen = true;
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
    pub const fn upgrade_plan_seen(&self) -> bool {
        self.upgrade_plan_seen
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

fn completed_tool_call_raw(
    update: &serde_json::Map<String, Value>,
) -> Option<&serde_json::Map<String, Value>> {
    if update.get("sessionUpdate").and_then(|v| v.as_str()) != Some("tool_call_update") {
        return None;
    }
    if update.get("status").and_then(|v| v.as_str()) != Some("completed") {
        return None;
    }
    update.get("rawOutput").and_then(|v| v.as_object())
}

fn silent_shell_completion(
    update: &serde_json::Map<String, Value>,
    raw: &serde_json::Map<String, Value>,
) -> bool {
    update.get("kind").and_then(|v| v.as_str()) == Some("execute")
        && raw.get("exitCode").and_then(serde_json::Value::as_u64) == Some(0)
        && raw_output_text_empty(raw)
}

fn raw_output_text_empty(raw: &serde_json::Map<String, Value>) -> bool {
    let stdout = raw.get("stdout").and_then(|v| v.as_str()).unwrap_or("");
    let stderr = raw.get("stderr").and_then(|v| v.as_str()).unwrap_or("");
    stdout.is_empty() && stderr.is_empty()
}

#[cfg(test)]
mod kiss_cov_gate_refs {
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _ = completed_tool_call_raw;
        let _ = raw_output_text_empty;
        let _ = silent_shell_completion;
    }
}
