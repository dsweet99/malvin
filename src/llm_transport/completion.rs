
use super::types::{CompletionResponse, ResponseUsage};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CompletionMeta {
    pub status: Option<u16>,
    pub body: Option<String>,
    pub usage: Option<ResponseUsage>,
    pub reasoning: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompletionOk {
    pub content: String,
    pub meta: CompletionMeta,
}

impl From<CompletionResponse> for CompletionOk {
    fn from(r: CompletionResponse) -> Self {
        Self {
            content: r.content,
            meta: CompletionMeta {
                usage: r.usage,
                reasoning: r.reasoning,
                ..CompletionMeta::default()
            },
        }
    }
}

impl CompletionOk {
    #[must_use]
    pub fn into_response(self) -> CompletionResponse {
        CompletionResponse {
            content: self.content,
            usage: self.meta.usage,
            reasoning: self.meta.reasoning,
        }
    }
}
