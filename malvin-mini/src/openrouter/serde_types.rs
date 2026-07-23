use serde::{Deserialize, Serialize};

use super::types::{ChatMessage, ResponseUsage};

#[derive(Serialize)]
pub(super) struct ChatCompletionRequest<'a> {
    pub model: &'a str,
    pub messages: &'a [ChatMessage],
    /// Caps completion length so providers that bill by reserved tokens stay within credit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

#[derive(Serialize, Deserialize)]
pub(super) struct ChatCompletionResponse {
    pub choices: Vec<ChatChoice>,
    pub usage: Option<ResponseUsage>,
}

#[derive(Serialize, Deserialize)]
pub(super) struct ChatChoice {
    pub message: Option<ChatChoiceMessage>,
}

#[derive(Serialize, Deserialize)]
pub(super) struct ChatChoiceMessage {
    pub content: Option<String>,
    pub reasoning: Option<String>,
    /// OpenRouter may return structured reasoning parts instead of a plain string.
    #[serde(default)]
    pub reasoning_details: Option<Vec<serde_json::Value>>,
}

impl ChatChoiceMessage {
    /// Assistant-visible content only. Do not promote reasoning into content: that
    /// makes thought tokens look like ordinary messages in tee logs.
    pub(super) fn text_content(&self) -> Option<String> {
        self.content.clone().filter(|text| !text.is_empty())
    }

    pub(super) fn reasoning_text(&self) -> Option<String> {
        if let Some(text) = self.reasoning.clone().filter(|t| !t.is_empty()) {
            return Some(text);
        }
        let details = self.reasoning_details.as_ref()?;
        let joined: Vec<String> = details
            .iter()
            .filter_map(|part| {
                part.get("text")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_string)
                    .or_else(|| {
                        part.get("summary")
                            .and_then(serde_json::Value::as_str)
                            .map(str::to_string)
                    })
            })
            .filter(|t| !t.is_empty())
            .collect();
        if joined.is_empty() {
            None
        } else {
            Some(joined.join("\n"))
        }
    }
}
