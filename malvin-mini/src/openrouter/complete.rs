use super::client::{build_request_headers, OpenRouterClient};
use super::serde_types::{ChatCompletionRequest, ChatCompletionResponse, ChatChoiceMessage};
use super::types::{ChatMessage, CompletionResponse};
use crate::error::OpenRouterError;
use crate::prompt_shrink::{is_prompt_too_long_error, shrink_messages};

impl OpenRouterClient {
    /// # Errors
    ///
    /// Returns [`OpenRouterError`] on HTTP or API failures.
    pub async fn complete(&self, messages: &[ChatMessage]) -> Result<CompletionResponse, OpenRouterError> {
        let mut current = messages.to_vec();
        loop {
            let url = format!(
                "{}/chat/completions",
                self.config().base_url.trim_end_matches('/')
            );
            let body = ChatCompletionRequest {
                model: &self.config().model,
                messages: &current,
            };
            let headers = build_request_headers(self.config())?;
            let resp = self.http().post(url).headers(headers).json(&body).send().await?;
            let status = resp.status().as_u16();
            let text = resp.text().await?;
            match map_http_status(status, &text) {
                Ok(()) => return parse_completion_body(&text),
                Err(err) if is_prompt_too_long_error(&err) => {
                    let shrunk = shrink_messages(&current);
                    if shrunk == current {
                        return Err(err);
                    }
                    current = shrunk;
                }
                Err(err) => return Err(err),
            }
        }
    }
}

fn map_http_status(status: u16, body: &str) -> Result<(), OpenRouterError> {
    match status {
        200 => Ok(()),
        401 => Err(OpenRouterError::Unauthorized {
            body: body.to_string(),
        }),
        402 | 403 => Err(OpenRouterError::BillingFailure { status, body: body.to_string() }),
        429 => Err(OpenRouterError::RateLimited {
            body: body.to_string(),
        }),
        500..=599 => Err(OpenRouterError::ServerError {
            status,
            body: body.to_string(),
        }),
        _ => Err(OpenRouterError::RequestFailed {
            status,
            body: body.to_string(),
        }),
    }
}

fn parse_completion_body(text: &str) -> Result<CompletionResponse, OpenRouterError> {
    let parsed: ChatCompletionResponse = serde_json::from_str(text)?;
    let content = parsed
        .choices
        .first()
        .and_then(|c| c.message.as_ref())
        .and_then(|m: &ChatChoiceMessage| m.content.clone())
        .ok_or(OpenRouterError::MissingContent)?;
    Ok(CompletionResponse {
        content,
        usage: parsed.usage,
    })
}

#[cfg(test)]
mod tests {
    use super::{map_http_status, parse_completion_body};
    use crate::error::OpenRouterError;

    #[test]
    fn map_http_status_maps_known_codes() {
        assert!(map_http_status(200, "").is_ok());
        assert!(matches!(
            map_http_status(401, "bad").unwrap_err(),
            OpenRouterError::Unauthorized { .. }
        ));
        assert!(matches!(
            map_http_status(429, "slow").unwrap_err(),
            OpenRouterError::RateLimited { .. }
        ));
        assert!(matches!(
            map_http_status(500, "boom").unwrap_err(),
            OpenRouterError::ServerError { .. }
        ));
        assert!(matches!(
            map_http_status(418, "teapot").unwrap_err(),
            OpenRouterError::RequestFailed { status: 418, .. }
        ));
    }

    #[test]
    fn parse_completion_body_extracts_content_and_usage() {
        let body = r#"{"choices":[{"message":{"content":"ok"}}],"usage":{"total_tokens":3}}"#;
        let resp = parse_completion_body(body).expect("parse");
        assert_eq!(resp.content, "ok");
        assert_eq!(resp.usage.and_then(|u| u.total_tokens), Some(3));
        let err = parse_completion_body(r#"{"choices":[{"message":{}}]}"#).expect_err("missing");
        assert!(matches!(err, OpenRouterError::MissingContent));
    }
}
