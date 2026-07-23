use super::client::{build_request_headers, OpenRouterClient};
use super::serde_types::ChatCompletionRequest;
use super::http_exchange::{CompletionWithMeta, HttpExchangeMeta};
use super::types::{ChatMessage, ChatRole};
use crate::error::OpenRouterError;

#[path = "complete_parse.rs"]
mod complete_parse;

pub(crate) use complete_parse::{map_http_status, outcome_from_http_body};

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
    ///
    /// When the provider rejects the reserved `max_tokens` for credit reasons, retries once
    /// with the affordable token cap parsed from the error body.
    ///
    /// When the body is HTTP 200 but assistant `content` is empty because completion hit
    /// `finish_reason=length` (common when reasoning tokens consume the cap), retries once
    /// with a higher `max_tokens`.
    pub async fn complete(&self, messages: &[ChatMessage]) -> CompletionWithMeta {
        let messages = with_tool_use_system_reminder(messages);
        let mut max_tokens = self.config().max_tokens;
        let first = self.complete_with_max_tokens(&messages, max_tokens).await;
        if let Some(afford) = affordable_max_tokens_from_outcome(&first)
            && max_tokens.is_none_or(|requested| afford < requested)
            && afford > 0
        {
            max_tokens = Some(afford);
            return self.complete_with_max_tokens(&messages, max_tokens).await;
        }
        if let Some(bumped) = length_truncated_max_tokens_bump(&first, max_tokens) {
            return self.complete_with_max_tokens(&messages, Some(bumped)).await;
        }
        first
    }

    async fn complete_with_max_tokens(
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
            model: &self.config().model,
            messages,
            max_tokens,
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

fn affordable_max_tokens_from_outcome(outcome: &CompletionWithMeta) -> Option<u32> {
    let err = outcome.result.as_ref().err()?;
    if !err.is_billing_failure() {
        return None;
    }
    parse_affordable_max_tokens(&err.to_string())
}

fn parse_affordable_max_tokens(text: &str) -> Option<u32> {
    let lower = text.to_ascii_lowercase();
    let key = "can only afford ";
    let start = lower.find(key)? + key.len();
    let digits: String = lower[start..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse::<u32>().ok().filter(|n| *n > 0)
}

/// If MissingContent came from a length-truncated completion, propose a larger cap.
fn length_truncated_max_tokens_bump(
    outcome: &CompletionWithMeta,
    current: Option<u32>,
) -> Option<u32> {
    let err = outcome.result.as_ref().err()?;
    if !matches!(err, OpenRouterError::MissingContent) {
        return None;
    }
    let body = outcome.http.body.as_deref()?;
    if !finish_reason_is_length(body) {
        return None;
    }
    let base = current.unwrap_or(4096);
    let bumped = base.saturating_mul(2).clamp(8192, 32_768);
    (bumped > base).then_some(bumped)
}

fn finish_reason_is_length(body: &str) -> bool {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(body) else {
        return body.to_ascii_lowercase().contains("\"finish_reason\":\"length\"");
    };
    value
        .pointer("/choices/0/finish_reason")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|reason| reason.eq_ignore_ascii_case("length"))
}

/// Short, domain-agnostic reminder: keep orientation current and finish unpaid probes.
/// Avoid tool-syntax prescriptions here; those belong in the harness, not the model preamble.
fn with_tool_use_system_reminder(messages: &[ChatMessage]) -> Vec<ChatMessage> {
    const REMINDER: &str = "State the problem and rival readings before acting. \
Prefer short, targeted trials. Study each outcome against a prior prediction. \
Do not stop while unpaid operational probes remain. Stay inside the working \
context named in the request.";
    if messages
        .first()
        .is_some_and(|m| matches!(m.role, ChatRole::System))
    {
        return messages.to_vec();
    }
    let mut out = Vec::with_capacity(messages.len() + 1);
    out.push(ChatMessage {
        role: ChatRole::System,
        content: REMINDER.to_string(),
    });
    out.extend_from_slice(messages);
    out
}

#[cfg(test)]
#[path = "complete_kiss_witness.rs"]
mod complete_kiss_witness;
