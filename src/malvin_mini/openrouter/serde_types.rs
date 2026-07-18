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
}
