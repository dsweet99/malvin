//! Retry loop implementation for `OpenRouter` HTTP completions.

use malvin_mini::{CompletionResponse, OpenRouterError};

use super::loop_http::{
    backoff_before_http_retry, exhaustion_message, HttpCompletionError, HttpRetryRequest,
};
use super::loop_mock::LlmCompletionOutcome;
use crate::agent_backend::mini::acp_trace_shim::MiniHttpExchangeRecord;
use crate::agent_backend::mini::trace::{record_http_exchange, MiniTraceSink};
use crate::nested_budget_scopes::BudgetScopeLayer;

#[derive(Copy, Clone)]
pub(crate) struct HttpRetryLimits {
    transport: u32,
}

pub(crate) struct HttpRetryCounters {
    transport_failures: u32,
    last_error: String,
}

enum HttpRetryStep {
    Stop(HttpCompletionError),
    Backoff,
}

impl HttpRetryCounters {
    const fn attempt(&self) -> u32 {
        self.transport_failures + 1
    }

    fn next(&mut self, err: &OpenRouterError, limits: HttpRetryLimits) -> HttpRetryStep {
        if err.is_context_overflow() {
            return HttpRetryStep::Stop(HttpCompletionError::ContextOverflow);
        }
        self.last_error = err.to_string();
        if err.is_billing_failure()
            && BudgetScopeLayer::MiniTransportRetry.billing_fails_immediately()
        {
            return HttpRetryStep::Stop(HttpCompletionError::Exhausted(exhaustion_message(
                1,
                limits.transport,
                &self.last_error,
            )));
        }
        self.transport_failures += 1;
        if self.transport_failures >= limits.transport {
            return HttpRetryStep::Stop(HttpCompletionError::Exhausted(exhaustion_message(
                self.transport_failures,
                limits.transport,
                &self.last_error,
            )));
        }
        HttpRetryStep::Backoff
    }
}

fn retry_limits(max_transport_retries: u32, single_attempt: bool) -> HttpRetryLimits {
    HttpRetryLimits {
        transport: BudgetScopeLayer::MiniTransportRetry
            .effective_max_attempts(max_transport_retries, single_attempt),
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
        max_transport_retries,
        single_attempt,
        timing,
        trace,
    } = req;
    let limits = retry_limits(max_transport_retries, single_attempt);
    let mut counters = HttpRetryCounters {
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
                HttpRetryStep::Backoff => {
                    backoff_before_http_retry(timing, counters.transport_failures, &err).await;
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
        let limits = retry_limits(2, false);
        let HttpRetryLimits { transport } = limits;
        assert_eq!(transport, 2);
        let single = retry_limits(9, true);
        let HttpRetryLimits {
            transport: single_transport,
        } = single;
        assert_eq!(single_transport, 1);
        let counters = HttpRetryCounters {
            transport_failures: 2,
            last_error: "err".into(),
        };
        assert_eq!(counters.attempt(), 3);
        let _ = std::mem::size_of::<HttpRetryStep>();
    }

    #[test]
    fn kiss_witness_http_retry_counter_next_paths() {
        let limits = HttpRetryLimits { transport: 2 };
        let mut rate_limited = HttpRetryCounters {
            transport_failures: 0,
            last_error: String::new(),
        };
        assert!(matches!(
            rate_limited.next(
                &OpenRouterError::RateLimited {
                    body: "429".into()
                },
                limits
            ),
            HttpRetryStep::Backoff
        ));
        let mut json = HttpRetryCounters {
            transport_failures: 0,
            last_error: String::new(),
        };
        let json_err = serde_json::from_str::<serde_json::Value>("not-json").unwrap_err();
        assert!(matches!(
            json.next(&OpenRouterError::Json(json_err), limits),
            HttpRetryStep::Backoff
        ));
        let mut overflow = HttpRetryCounters {
            transport_failures: 0,
            last_error: String::new(),
        };
        assert!(matches!(
            overflow.next(
                &OpenRouterError::ContextOverflow {
                    body: "overflow".into(),
                    message_count: 1,
                },
                limits
            ),
            HttpRetryStep::Stop(HttpCompletionError::ContextOverflow)
        ));
        let mut billing = HttpRetryCounters {
            transport_failures: 0,
            last_error: String::new(),
        };
        assert!(matches!(
            billing.next(
                &OpenRouterError::BillingFailure {
                    status: 402,
                    body: "no credits".into(),
                },
                limits
            ),
            HttpRetryStep::Stop(HttpCompletionError::Exhausted(_))
        ));
    }
}
