use crate::openrouter_transport::types::CompletionResponse;
use crate::llm_transport::TransportError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpExchangeMeta {
    pub status: Option<u16>,
    pub body: Option<String>,
}

#[derive(Debug)]
pub struct CompletionWithMeta {
    pub result: Result<CompletionResponse, TransportError>,
    pub http: HttpExchangeMeta,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm_transport::TransportError;

    use crate::openrouter_transport::types::CompletionResponse;

    #[test]
    fn completion_with_meta_exposes_result_and_http() {
        let ok = CompletionWithMeta {
            result: Ok(CompletionResponse {
                content: "ok".into(),
                usage: None,
                reasoning: None,
            }),
            http: HttpExchangeMeta {
                status: Some(200),
                body: Some("body".into()),
            },
        };
        assert_eq!(ok.result.as_ref().expect("ok").content, "ok");
        assert_eq!(ok.http.body.as_deref(), Some("body"));
        let err = CompletionWithMeta {
            result: Err(TransportError::MissingContent),
            http: HttpExchangeMeta {
                status: None,
                body: None,
            },
        };
        assert!(err.result.is_err());
        assert!(err.http.status.is_none());
    }
}
