use super::client::{build_request_headers, OpenRouterClient};
use super::serde_types::ChatCompletionRequest;
use super::http_exchange::{CompletionWithMeta, HttpExchangeMeta};
use super::types::ChatMessage;
use crate::llm_transport::TransportError;

pub(crate) use super::complete_parse::{map_http_status, outcome_from_http_body};

pub(crate) fn completion_with_meta(
    result: Result<super::types::CompletionResponse, TransportError>,
    http: HttpExchangeMeta,
) -> CompletionWithMeta {
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
        Err(TransportError::Network(err.to_string())),
        transport_meta(status, None),
    )
}

pub(crate) fn completion_post_url(base_url: &str) -> String {
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
    /// HTTP/provider completion with credit and length-cap retries only.
    /// Protocol-shape / shrink recovery lives in the Mini agent (`mini_agent::protocol`).
    pub async fn complete_http(&self, messages: &[ChatMessage]) -> CompletionWithMeta {
        let mut max_tokens = self.config().max_tokens;
        loop {
            let outcome = self.complete_with_max_tokens(messages, max_tokens).await;
            if let Some(afford) = affordable_max_tokens_from_outcome(&outcome)
                && max_tokens.is_none_or(|requested| afford < requested)
                && afford > 0
            {
                max_tokens = Some(afford);
                return self.complete_with_max_tokens(messages, max_tokens).await;
            }
            if let Some(bumped) = length_truncated_max_tokens_bump(&outcome, max_tokens) {
                max_tokens = Some(bumped);
                continue;
            }
            return outcome;
        }
    }

    /// Compatibility alias for [`Self::complete_http`] (protocol-shape retries live in Mini).
    pub async fn complete(&self, messages: &[ChatMessage]) -> CompletionWithMeta {
        self.complete_http(messages).await
    }

    pub(crate) async fn complete_with_max_tokens(
        &self,
        messages: &[ChatMessage],
        max_tokens: Option<u32>,
    ) -> CompletionWithMeta {
        match self.fetch_completion_body(messages, max_tokens).await {
            Ok((status, text)) => outcome_from_http_body(status, text, messages.len()),
            Err(meta) => meta,
        }
    }

    pub(crate) async fn fetch_completion_body(
        &self,
        messages: &[ChatMessage],
        max_tokens: Option<u32>,
    ) -> Result<(u16, String), CompletionWithMeta> {
        let url = completion_post_url(&self.config().base_url);
        let body = ChatCompletionRequest {
            model: self.config().model.as_str(),
            messages,
            max_tokens,
        };
        let headers = match build_request_headers(self.config()) {
            Ok(h) => h,
            Err(err) => {
                return Err(completion_with_meta(
                    Err(err),
                    transport_meta(None, None),
                ));
            }
        };
        let response = post_chat_completion(self, url, &body, headers).await?;
        let status = response.status().as_u16();
        match response.text().await {
            Ok(text) => Ok((status, text)),
            Err(err) => Err(transport_failure_meta(Some(status), err)),
        }
    }
}

pub(crate) fn affordable_max_tokens_from_outcome(outcome: &CompletionWithMeta) -> Option<u32> {
    let Err(TransportError::BillingFailure { body, .. }) = outcome.result.as_ref() else {
        return None;
    };
    parse_affordable_max_tokens(body)
}

pub(crate) fn parse_affordable_max_tokens(text: &str) -> Option<u32> {
    const NEEDLE: &str = "can only afford ";
    let start = text.find(NEEDLE)? + NEEDLE.len();
    let digits: String = text[start..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

pub(crate) fn finish_reason_is_length(body: &str) -> bool {
    body.contains("\"finish_reason\":\"length\"") || body.contains("\"finish_reason\": \"length\"")
}

pub(crate) fn length_truncated_max_tokens_bump(
    outcome: &CompletionWithMeta,
    current: Option<u32>,
) -> Option<u32> {
    let missing = matches!(outcome.result, Err(TransportError::MissingContent));
    if !missing {
        return None;
    }
    let body = outcome.http.body.as_deref()?;
    if !finish_reason_is_length(body) {
        return None;
    }
    if body_has_reasoning(body) {
        return None;
    }
    let base = current.unwrap_or(4096);
    let bumped = base.saturating_mul(2).clamp(4096, 8192);
    (bumped > base).then_some(bumped)
}

fn body_has_reasoning(body: &str) -> bool {
    body.contains("\"reasoning\"") || body.contains("\"reasoning_details\"")
}
