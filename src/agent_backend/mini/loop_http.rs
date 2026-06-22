//! HTTP completion retries for the mini loop driver.

use std::sync::{Arc, Mutex};

use malvin_mini::{ChatMessage, CompletionResponse};

use super::loop_mock::LlmBackend;
use crate::acp::AgentError;

pub struct HttpRetryRequest<'a> {
    pub llm: &'a LlmBackend,
    pub messages: &'a [ChatMessage],
    pub max_retries: u32,
    pub single_attempt: bool,
    pub timing: Option<&'a Arc<Mutex<crate::run_timing::RunTiming>>>,
}

pub async fn complete_with_http_retries(req: HttpRetryRequest<'_>) -> Result<CompletionResponse, AgentError> {
    let HttpRetryRequest {
        llm,
        messages,
        max_retries,
        single_attempt,
        timing,
    } = req;
    let max_attempts = if single_attempt { 1 } else { max_retries.max(1) };
    let mut last_error = String::new();
    for attempt in 1..=max_attempts {
        match llm.complete(messages).await {
            Ok(r) => return Ok(r),
            Err(e) => {
                last_error = e.to_string();
                if !e.is_retryable() || attempt >= max_attempts {
                    break;
                }
                let sleep = if attempt == 1 {
                    std::time::Duration::from_secs(1)
                } else {
                    std::time::Duration::from_secs(3)
                };
                crate::run_timing::record_backoff(timing, sleep);
                crate::acp::agent_backoff_sleep(sleep).await;
            }
        }
    }
    Err(AgentError(format!(
        "http_completion: OpenRouter failed after {max_attempts} attempts. Last error:\n{last_error}"
    )))
}

#[cfg(test)]
mod tests {
    use malvin_mini::{ChatMessage, ChatRole, CompletionResponse};

    use super::{complete_with_http_retries, HttpRetryRequest};
    use super::super::loop_mock::{LlmBackend, MockScript, MockStep};

    #[tokio::test]
    async fn complete_with_http_retries_succeeds_on_second_mock_attempt() {
        let llm = LlmBackend::Mock(std::sync::Mutex::new(MockScript {
            responses: vec![
                MockStep::RateLimited,
                MockStep::Ok(CompletionResponse {
                    content: "ok".into(),
                    usage: None,
                }),
            ],
            call_count: 0,
            on_response: None,
        }));
        let messages = [ChatMessage {
            role: ChatRole::User,
            content: "hi".into(),
        }];
        let resp = complete_with_http_retries(HttpRetryRequest {
            llm: &llm,
            messages: &messages,
            max_retries: 2,
            single_attempt: false,
            timing: None,
        })
        .await
        .expect("retry ok");
        assert_eq!(resp.content, "ok");
    }
}
#[cfg(test)]
#[path = "loop_http_test.rs"]
mod loop_http_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<HttpRetryRequest> = None;
        let _ = complete_with_http_retries;
    }
}
