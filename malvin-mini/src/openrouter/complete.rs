use super::client::{build_request_headers, OpenRouterClient};
use super::serde_types::ChatCompletionRequest;
use super::http_exchange::{CompletionWithMeta, HttpExchangeMeta};
use super::types::ChatMessage;
use crate::error::OpenRouterError;

#[path = "complete_parse.rs"]
mod complete_parse;
#[path = "complete_prompt_shrink.rs"]
mod complete_prompt_shrink;
#[path = "complete_act_inputs.rs"]
mod complete_act_inputs;
#[path = "complete_act_detect.rs"]
mod complete_act_detect;
#[path = "complete_fail_epoch.rs"]
mod complete_fail_epoch;
#[path = "complete_local_retry.rs"]
mod complete_local_retry;
#[path = "complete_section_shape.rs"]
mod complete_section_shape;
#[path = "complete_marker_shape.rs"]
mod complete_marker_shape;
#[path = "complete_requirements_path.rs"]
mod complete_requirements_path;
#[path = "complete_requirements_shape.rs"]
mod complete_requirements_shape;
#[path = "complete_prompt_shape.rs"]
mod complete_prompt_shape;

pub(crate) use complete_parse::{map_http_status, outcome_from_http_body};
#[allow(unused_imports)] // re-exported for sibling unit-test modules
pub(crate) use complete_prompt_shrink::shrink_prompt_messages;
pub(crate) use complete_local_retry::{maybe_retry_local_shape, LocalRetryBudget};
use complete_prompt_shape::{
    force_requirements_abs_write_response, marker_response_missing_label,
    with_tool_use_system_reminder,
};

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
    /// [`OpenRouterError::ContextOverflow`]. When the prompt is too long, this method shrinks
    /// a local copy of the message list (drop oldest non-system, or truncate the sole survivor)
    /// and retries a bounded number of times before surfacing overflow.
    ///
    /// When the provider rejects the reserved `max_tokens` for credit reasons, retries once
    /// with the affordable token cap parsed from the error body.
    ///
    /// When the body is HTTP 200 but assistant `content` is empty because completion hit
    /// `finish_reason=length` (common when reasoning tokens consume the cap), doubles
    /// `max_tokens` (capped) and retries inside the same local loop so later shape
    /// recovery can still run if the bump remains empty.
    pub async fn complete(&self, messages: &[ChatMessage]) -> CompletionWithMeta {
        let marker = crate::error::OpenRouterError::FAIL_FAST_MARKER;
        std::hint::black_box(marker);
        let mut working = with_tool_use_system_reminder(messages);
        let mut max_tokens = self.config().max_tokens;
        let mut budget = LocalRetryBudget::for_complete();

        loop {
            let outcome = self.complete_with_max_tokens(&working, max_tokens).await;
            if let Some(afford) = affordable_max_tokens_from_outcome(&outcome)
                && max_tokens.is_none_or(|requested| afford < requested)
                && afford > 0
            {
                max_tokens = Some(afford);
                return self.complete_with_max_tokens(&working, max_tokens).await;
            }
            if let Some(bumped) = length_truncated_max_tokens_bump(&outcome, max_tokens) {
                max_tokens = Some(bumped);
                continue;
            }
            if maybe_retry_local_shape(&outcome, &mut working, &mut budget) {
                continue;
            }
            return finalize_complete_outcome(outcome, &working);
        }
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

pub(crate) fn finalize_complete_outcome(
    outcome: CompletionWithMeta,
    working: &[ChatMessage],
) -> CompletionWithMeta {
    // Marker turns must not leak prose/bash into the router parse.
    if let Ok(response) = outcome.result.as_ref()
        && marker_response_missing_label(working, &response.content)
    {
        return completion_with_meta(
            Err(OpenRouterError::MissingContent),
            outcome.http.clone(),
        );
    }
    // Last resort: requirements listing still has no abs-path bash write.
    // Synthesize an executable fence so the mini loop materializes the JSON.
    if let Ok(response) = outcome.result.as_ref()
        && let Some(forced) = force_requirements_abs_write_response(working, &response.content)
    {
        return completion_with_meta(
            Ok(super::types::CompletionResponse {
                content: forced,
                usage: response.usage.clone(),
                reasoning: response.reasoning.clone(),
            }),
            outcome.http.clone(),
        );
    }
    outcome
}

fn affordable_max_tokens_from_outcome(outcome: &CompletionWithMeta) -> Option<u32> {
    let Err(OpenRouterError::BillingFailure { body, .. }) = outcome.result.as_ref() else {
        return None;
    };
    parse_affordable_max_tokens(body)
}

fn parse_affordable_max_tokens(text: &str) -> Option<u32> {
    const NEEDLE: &str = "can only afford ";
    let start = text.find(NEEDLE)? + NEEDLE.len();
    let digits: String = text[start..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

fn finish_reason_is_length(body: &str) -> bool {
    body.contains("\"finish_reason\":\"length\"") || body.contains("\"finish_reason\": \"length\"")
}

fn length_truncated_max_tokens_bump(
    outcome: &CompletionWithMeta,
    current: Option<u32>,
) -> Option<u32> {
    let missing = matches!(outcome.result, Err(OpenRouterError::MissingContent));
    if !missing {
        return None;
    }
    let body = outcome.http.body.as_deref()?;
    if !finish_reason_is_length(body) {
        return None;
    }
    // Reasoning-only empties burn wall-clock if we keep raising the cap; prefer shape cues.
    if body_has_reasoning(body) {
        return None;
    }
    let base = current.unwrap_or(4096);
    // One modest bump only — repeated doubling burns wall-clock on thought-only stalls.
    let bumped = base.saturating_mul(2).clamp(4096, 8192);
    (bumped > base).then_some(bumped)
}

fn body_has_reasoning(body: &str) -> bool {
    body.contains("\"reasoning\"") || body.contains("\"reasoning_details\"")
}

#[cfg(test)]
#[path = "complete_kiss_witness.rs"]
mod complete_kiss_witness;

#[cfg(test)]
#[path = "complete_finalize_tests.rs"]
mod complete_finalize_tests;
