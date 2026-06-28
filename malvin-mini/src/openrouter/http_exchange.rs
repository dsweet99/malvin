use super::types::CompletionResponse;
use crate::error::OpenRouterError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpExchangeMeta {
    pub status: Option<u16>,
    pub body: Option<String>,
}

#[derive(Debug)]
pub struct CompletionWithMeta {
    pub result: Result<CompletionResponse, OpenRouterError>,
    pub http: HttpExchangeMeta,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::OpenRouterError;

    use super::super::types::CompletionResponse;

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
            result: Err(OpenRouterError::MissingContent),
            http: HttpExchangeMeta {
                status: None,
                body: None,
            },
        };
        assert!(err.result.is_err());
        assert!(err.http.status.is_none());
    }
}
