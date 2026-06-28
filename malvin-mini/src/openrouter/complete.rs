use super::client::{build_request_headers, OpenRouterClient};
use super::serde_types::ChatCompletionRequest;
use super::http_exchange::{CompletionWithMeta, HttpExchangeMeta};
use super::types::ChatMessage;
use crate::error::OpenRouterError;

#[path = "complete_parse.rs"]
mod complete_parse;

pub(crate) use complete_parse::outcome_from_http_body;

pub(crate) fn completion_with_meta(result: Result<super::types::CompletionResponse, OpenRouterError>, http: HttpExchangeMeta) -> CompletionWithMeta {
    CompletionWithMeta { result, http }
}

pub(crate) fn transport_meta(status: Option<u16>, body: Option<String>) -> HttpExchangeMeta {
    HttpExchangeMeta { status, body }
}

pub(crate) fn transport_failure_meta(
    status: Option<u16>,
    err: reqwest::Error,
) -> CompletionWithMeta {
    completion_with_meta(
        Err(OpenRouterError::Transport(err)),
        transport_meta(status, None),
    )
}

fn completion_post_url(base_url: &str) -> String {
    format!("{}/chat/completions", base_url.trim_end_matches('/'))
}

async fn post_chat_completion(
    client: &OpenRouterClient,
    url: String,
    body: &ChatCompletionRequest<'_>,
    headers: reqwest::header::HeaderMap,
) -> Result<reqwest::Response, CompletionWithMeta> {
    match client.http().post(url).headers(headers).json(body).send().await {
        Ok(response) => Ok(response),
        Err(err) => Err(transport_failure_meta(None, err)),
    }
}

impl OpenRouterClient {
    /// # Errors
    ///
    /// Returns [`OpenRouterError`] on HTTP or API failures. Context-length failures return
    /// [`OpenRouterError::ContextOverflow`] without mutating messages.
    pub async fn complete(&self, messages: &[ChatMessage]) -> CompletionWithMeta {
        match self.fetch_completion_body(messages).await {
            Ok((status, text)) => outcome_from_http_body(status, text, messages.len()),
            Err(meta) => meta,
        }
    }

    pub(crate) async fn fetch_completion_body(
        &self,
        messages: &[ChatMessage],
    ) -> Result<(u16, String), CompletionWithMeta> {
        let url = completion_post_url(&self.config().base_url);
        let body = ChatCompletionRequest {
            model: &self.config().model,
            messages,
        };
        let headers = match build_request_headers(self.config()) {
            Ok(h) => h,
            Err(e) => return Err(completion_with_meta(Err(e), transport_meta(None, None))),
        };
        let resp = match post_chat_completion(self, url, &body, headers).await {
            Ok(response) => response,
            Err(meta) => return Err(meta),
        };
        let status = resp.status().as_u16();
        match resp.text().await {
            Ok(text) => Ok((status, text)),
            Err(e) => Err(transport_failure_meta(Some(status), e)),
        }
    }
}

#[cfg(test)]
mod kiss_witness {
    use crate::error::OpenRouterError;

    use super::{completion_post_url, completion_with_meta, outcome_from_http_body, transport_failure_meta, transport_meta};

    #[test]
    fn kiss_witness_completion_post_url() {
        assert_eq!(
            completion_post_url("https://openrouter.ai/api/"),
            "https://openrouter.ai/api/chat/completions"
        );
    }

    #[test]
    fn completion_with_meta_and_transport_meta_helpers() {
        let http = transport_meta(Some(201), Some("body".into()));
        let wrapped = completion_with_meta(Err(OpenRouterError::MissingContent), http);
        assert_eq!(wrapped.http.status, Some(201));
        assert_eq!(wrapped.http.body.as_deref(), Some("body"));
        let ok = outcome_from_http_body(
            200,
            r#"{"choices":[{"message":{"content":"hi"}}]}"#.into(),
            1,
        );
        assert_eq!(ok.result.as_ref().expect("ok").content, "hi");
        let err = outcome_from_http_body(418, "teapot".into(), 1);
        assert!(err.result.is_err());
    }

    #[test]
    fn kiss_witness_transport_failure_meta() {
        let err = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime")
            .block_on(async {
                reqwest::Client::new()
                    .get("http://127.0.0.1:1")
                    .send()
                    .await
                    .expect_err("transport")
            });
        let none_status = transport_failure_meta(None, err);
        assert!(none_status.result.is_err());
        assert_eq!(none_status.http.status, None);
        let err2 = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime")
            .block_on(async {
                reqwest::Client::new()
                    .get("http://127.0.0.1:1")
                    .send()
                    .await
                    .expect_err("transport")
            });
        let with_status = transport_failure_meta(Some(200), err2);
        assert_eq!(with_status.http.status, Some(200));
    }
}
