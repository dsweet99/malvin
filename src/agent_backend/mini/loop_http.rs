//! HTTP completion retries for the mini loop driver.

use std::sync::{Arc, Mutex};

use malvin_mini::{ChatMessage, CompletionResponse};

use super::loop_mock::LlmBackend;

pub enum HttpCompletionError {
    Exhausted(String),
    ContextOverflow,
}

impl std::fmt::Debug for HttpCompletionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exhausted(msg) => f.debug_tuple("Exhausted").field(msg).finish(),
            Self::ContextOverflow => f.write_str("ContextOverflow"),
        }
    }
}

pub struct HttpRetryRequest<'a> {
    pub llm: &'a LlmBackend,
    pub messages: &'a [ChatMessage],
    pub max_retries: u32,
    pub single_attempt: bool,
    pub timing: Option<&'a Arc<Mutex<crate::run_timing::RunTiming>>>,
}

pub async fn complete_with_http_retries(
    req: HttpRetryRequest<'_>,
) -> Result<CompletionResponse, HttpCompletionError> {
    let HttpRetryRequest {
        llm,
        messages,
        max_retries,
        single_attempt,
        timing,
    } = req;
    let max_attempts = if single_attempt { 1 } else { max_retries.max(1) };
    let mut last_error = String::new();
    let mut attempts_made = 0_u32;
    for attempt in 1..=max_attempts {
        attempts_made = attempt;
        match llm.complete(messages).await {
            Ok(r) => return Ok(r),
            Err(e) => {
                if e.is_context_overflow() {
                    return Err(HttpCompletionError::ContextOverflow);
                }
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
    Err(HttpCompletionError::Exhausted(format!(
        "http_completion: OpenRouter failed after {attempts_made} attempts. Last error:\n{last_error}"
    )))
}

#[cfg(test)]
mod tests {
    use malvin_mini::{ChatMessage, ChatRole, CompletionResponse};

    use super::{complete_with_http_retries, HttpCompletionError, HttpRetryRequest};
    use super::super::loop_mock::{LlmBackend, MockScript, MockStep};

    #[tokio::test]
    async fn complete_with_http_retries_reports_actual_attempt_count() {
        let llm = LlmBackend::Mock(std::sync::Mutex::new(MockScript {
            responses: vec![MockStep::RequestFailed {
                status: 400,
                body: "bad".into(),
            }],
            call_count: 0,
            on_response: None,
        }));
        let messages = [ChatMessage {
            role: ChatRole::User,
            content: "hi".into(),
        }];
        let err = complete_with_http_retries(HttpRetryRequest {
            llm: &llm,
            messages: &messages,
            max_retries: 9999,
            single_attempt: false,
            timing: None,
        })
        .await
        .expect_err("400");
        assert!(matches!(
            err,
            HttpCompletionError::Exhausted(ref msg)
                if msg.contains("after 1 attempts") && !msg.contains("9999")
        ));
    }

    #[tokio::test]
    async fn complete_with_http_retries_succeeds_on_second_mock_attempt() {
        let llm = LlmBackend::Mock(std::sync::Mutex::new(MockScript {
            responses: vec![
                MockStep::RateLimited,
                MockStep::Ok(CompletionResponse {
                    content: "ok".into(),
                    usage: None,
                    reasoning: None,
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

    #[tokio::test]
    async fn complete_with_http_retries_maps_context_overflow() {
        let llm = LlmBackend::Mock(std::sync::Mutex::new(MockScript {
            responses: vec![MockStep::ContextOverflow],
            call_count: 0,
            on_response: None,
        }));
        let messages = [ChatMessage {
            role: ChatRole::User,
            content: "hi".into(),
        }];
        let err = complete_with_http_retries(HttpRetryRequest {
            llm: &llm,
            messages: &messages,
            max_retries: 1,
            single_attempt: true,
            timing: None,
        })
        .await
        .expect_err("overflow");
        assert!(matches!(err, HttpCompletionError::ContextOverflow));
    }
}
