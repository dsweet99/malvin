use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseUsage {
    pub prompt_tokens: Option<u64>,
    pub completion_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
    pub cost: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompletionResponse {
    pub content: String,
    pub usage: Option<ResponseUsage>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kiss_witness_chat_role() {
        let _roles = [ChatRole::System, ChatRole::User, ChatRole::Assistant];
        let _ = _roles;
    }

    #[test]
    fn kiss_witness_chat_message() {
        let msg = ChatMessage {
            role: ChatRole::User,
            content: String::new(),
        };
        let ChatMessage { role, content } = msg;
        let _ = (role, content);
    }

    #[test]
    fn kiss_witness_response_usage() {
        let usage = ResponseUsage {
            prompt_tokens: None,
            completion_tokens: None,
            total_tokens: None,
            cost: None,
        };
        let ResponseUsage {
            prompt_tokens,
            completion_tokens,
            total_tokens,
            cost,
        } = usage;
        let _ = (prompt_tokens, completion_tokens, total_tokens, cost);
    }

    #[test]
    fn kiss_witness_completion_response() {
        let resp = CompletionResponse {
            content: String::new(),
            usage: None,
        };
        let CompletionResponse { content, usage } = resp;
        let _ = (content, usage);
    }
}
#[cfg(test)]
#[path = "types_test.rs"]
mod types_test;
#[cfg(test)]
#[path = "types_kiss_cov_test.rs"]
mod types_kiss_cov_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<ChatMessage> = None;
        let _: Option<ChatRole> = None;
        let _: Option<CompletionResponse> = None;
    }
}
