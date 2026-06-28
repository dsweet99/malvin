use serde::{Deserialize, Serialize};

use super::types::{ChatMessage, ResponseUsage};

#[derive(Serialize)]
pub(super) struct ChatCompletionRequest<'a> {
    pub model: &'a str,
    pub messages: &'a [ChatMessage],
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
}

impl ChatChoiceMessage {
    pub(super) fn text_content(&self) -> Option<String> {
        self.content
            .clone()
            .filter(|text| !text.is_empty())
            .or_else(|| self.reasoning.clone())
    }
}
