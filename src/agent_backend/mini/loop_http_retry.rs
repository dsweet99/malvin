//! Retry loop implementation for `OpenRouter` HTTP completions.

use malvin_mini::{CompletionResponse, OpenRouterError};

use super::loop_http::{
    backoff_before_http_retry, exhaustion_message, HttpCompletionError, HttpRetryRequest, RetryClass,
};
use super::loop_mock::LlmCompletionOutcome;
use crate::agent_backend::mini::acp_trace_shim::MiniHttpExchangeRecord;
use crate::agent_backend::mini::trace::{record_http_exchange, MiniTraceSink};

#[derive(Copy, Clone)]
pub(crate) struct HttpRetryLimits {
    api: u32,
    transport: u32,
}

pub(crate) struct HttpRetryCounters {
    api_failures: u32,
    transport_failures: u32,
    last_error: String,
}

enum HttpRetryStep {
    Stop(HttpCompletionError),
    Backoff(RetryClass),
}

impl HttpRetryCounters {
    const fn attempt(&self) -> u32 {
        self.api_failures + self.transport_failures + 1
    }

    fn next(&mut self, err: &OpenRouterError, limits: HttpRetryLimits) -> HttpRetryStep {
        if err.is_context_overflow() {
            return HttpRetryStep::Stop(HttpCompletionError::ContextOverflow);
        }
        self.last_error = err.to_string();
        if err.is_retryable() {
            self.api_failures += 1;
            if self.api_failures >= limits.api {
                return HttpRetryStep::Stop(HttpCompletionError::Exhausted(exhaustion_message(
                    RetryClass::Api,
                    self.api_failures,
                    limits.api,
                    &self.last_error,
                )));
            }
            return HttpRetryStep::Backoff(RetryClass::Api);
        }
        if err.is_transport_retryable() {
            self.transport_failures += 1;
            if self.transport_failures >= limits.transport {
                return HttpRetryStep::Stop(HttpCompletionError::Exhausted(exhaustion_message(
                    RetryClass::Transport,
                    self.transport_failures,
                    limits.transport,
                    &self.last_error,
                )));
            }
            return HttpRetryStep::Backoff(RetryClass::Transport);
        }
        HttpRetryStep::Stop(HttpCompletionError::Exhausted(exhaustion_message(
            RetryClass::Api,
            self.api_failures.max(1),
            limits.api,
            &self.last_error,
        )))
    }
}

fn retry_limits(max_api_retries: u32, max_transport_retries: u32, single_attempt: bool) -> HttpRetryLimits {
    if single_attempt {
        HttpRetryLimits {
            api: 1,
            transport: 1,
        }
    } else {
        HttpRetryLimits {
            api: max_api_retries.max(1),
            transport: max_transport_retries.max(1),
        }
    }
}

fn record_outcome(trace: Option<&MiniTraceSink>, attempt: u32, outcome: &LlmCompletionOutcome) {
    let Some(sink) = trace else {
        return;
    };
    record_http_exchange(
        sink,
        MiniHttpExchangeRecord {
            attempt,
            status: outcome.http.status,
            body: outcome.http.body.as_deref(),
            error: outcome.result.as_ref().err().map(ToString::to_string),
        },
    );
}

pub async fn complete_with_http_retries(
    req: HttpRetryRequest<'_>,
) -> Result<CompletionResponse, HttpCompletionError> {
    let HttpRetryRequest {
        llm,
        messages,
        max_api_retries,
        max_transport_retries,
        single_attempt,
        timing,
        trace,
    } = req;
    let limits = retry_limits(max_api_retries, max_transport_retries, single_attempt);
    let mut counters = HttpRetryCounters {
        api_failures: 0,
        transport_failures: 0,
        last_error: String::new(),
    };
    loop {
        let outcome = llm.complete(messages).await;
        record_outcome(trace, counters.attempt(), &outcome);
        match outcome.result {
            Ok(response) => return Ok(response),
            Err(err) => match counters.next(&err, limits) {
                HttpRetryStep::Stop(stop) => return Err(stop),
                HttpRetryStep::Backoff(class) => {
                    let failures = match class {
                        RetryClass::Api => counters.api_failures,
                        RetryClass::Transport => counters.transport_failures,
                    };
                    backoff_before_http_retry(timing, class, failures, &err).await;
                }
            },
        }
    }
}

#[cfg(test)]
mod kiss_witness {
    use super::*;

    #[test]
    fn kiss_witness_http_retry_limits_and_counters() {
        let limits = retry_limits(3, 2, false);
        let HttpRetryLimits { api, transport } = limits;
        assert_eq!(api, 3);
        assert_eq!(transport, 2);
        let single = retry_limits(9, 9, true);
        let HttpRetryLimits {
            api: single_api,
            transport: single_transport,
        } = single;
        assert_eq!(single_api, 1);
        assert_eq!(single_transport, 1);
        let counters = HttpRetryCounters {
            api_failures: 1,
            transport_failures: 2,
            last_error: "err".into(),
        };
        assert_eq!(counters.attempt(), 4);
        let _ = std::mem::size_of::<HttpRetryStep>();
    }

    #[test]
    fn kiss_witness_http_retry_counter_next_paths() {
        let limits = HttpRetryLimits {
            api: 2,
            transport: 2,
        };
        let mut counters = HttpRetryCounters {
            api_failures: 0,
            transport_failures: 0,
            last_error: String::new(),
        };
        assert!(matches!(
            counters.next(
                &OpenRouterError::RateLimited {
                    body: "429".into()
                },
                limits
            ),
            HttpRetryStep::Backoff(RetryClass::Api)
        ));
        let json_err = serde_json::from_str::<serde_json::Value>("not-json").unwrap_err();
        assert!(matches!(
            counters.next(&OpenRouterError::Json(json_err), limits),
            HttpRetryStep::Backoff(RetryClass::Transport)
        ));
        assert!(matches!(
            counters.next(
                &OpenRouterError::ContextOverflow {
                    body: "overflow".into(),
                    message_count: 1,
                },
                limits
            ),
            HttpRetryStep::Stop(HttpCompletionError::ContextOverflow)
        ));
    }
}
