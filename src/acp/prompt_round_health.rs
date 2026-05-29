//! Per-prompt summary of ACP stream signals observed during a live `session/prompt`.

use serde_json::Value;

const UPGRADE_PLAN_WINDOW: usize = 128;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PromptRoundHealth {
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

    #[must_use]
    pub const fn upgrade_plan_seen(&self) -> bool {
        self.upgrade_plan_seen
    }
}
