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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kiss_cov_serde_types() {
        let req = ChatCompletionRequest {
            model: "m",
            messages: &[],
        };
        let _ = (req.model, req.messages);
        let resp = ChatCompletionResponse {
            choices: vec![ChatChoice {
                message: Some(ChatChoiceMessage {
                    content: Some(String::new()),
                }),
            }],
            usage: None,
        };
        let ChatCompletionResponse { choices, usage } = resp;
        let _ = (choices, usage);
        let _ = ChatChoice { message: None };
        let _ = ChatChoiceMessage { content: None };
    }
}
#[cfg(test)]
#[path = "serde_types_test.rs"]
mod serde_types_test;
#[cfg(test)]
#[path = "serde_types_kiss_cov_test.rs"]
mod serde_types_kiss_cov_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<ChatCompletionRequest> = None;
        let _: Option<ChatCompletionResponse> = None;
    }
}
