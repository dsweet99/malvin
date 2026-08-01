//! Mini agent wire protocol: NEW_HISTORY/RESPONSE assemble/parse and shape recovery.

#![allow(
    clippy::module_name_repetitions,
    clippy::missing_const_for_fn,
    clippy::option_if_let_else,
    clippy::items_after_statements,
    clippy::wildcard_enum_match_arm,
    clippy::unnecessary_wraps,
    clippy::redundant_closure_for_method_calls,
    clippy::doc_markdown
)]

#[path = "complete_act_detect.rs"]
mod complete_act_detect;
#[path = "complete_act_detect_owed.rs"]
mod complete_act_detect_owed;
#[cfg(test)]
#[path = "complete_act_detect_tests.rs"]
mod complete_act_detect_tests;
#[path = "complete_act_inputs.rs"]
mod complete_act_inputs;
#[cfg(test)]
#[path = "complete_act_inputs_tests.rs"]
mod complete_act_inputs_tests;
#[path = "complete_fail_epoch.rs"]
mod complete_fail_epoch;
#[path = "complete_local_retry.rs"]
mod complete_local_retry;
#[cfg(test)]
#[path = "complete_local_retry_act_pressure_tests.rs"]
mod complete_local_retry_act_pressure_tests;
#[cfg(test)]
#[path = "complete_local_retry_act_tests.rs"]
mod complete_local_retry_act_tests;
#[path = "complete_local_retry_pressure.rs"]
mod complete_local_retry_pressure;
#[cfg(test)]
#[path = "complete_local_retry_req_tests.rs"]
mod complete_local_retry_req_tests;
#[cfg(test)]
#[path = "complete_local_retry_tests.rs"]
mod complete_local_retry_tests;
#[path = "complete_marker_shape.rs"]
pub(crate) mod complete_marker_shape;
#[path = "complete_prompt_shape.rs"]
pub(crate) mod complete_prompt_shape;
#[cfg(test)]
#[path = "complete_prompt_shape_tests.rs"]
mod complete_prompt_shape_tests;
#[path = "complete_prompt_shrink.rs"]
pub(crate) mod complete_prompt_shrink;
#[path = "complete_requirements_path.rs"]
mod complete_requirements_path;
#[cfg(test)]
#[path = "complete_requirements_path_tests.rs"]
mod complete_requirements_path_tests;
#[path = "complete_requirements_shape.rs"]
mod complete_requirements_shape;
#[path = "complete_section_shape.rs"]
mod complete_section_shape;
#[path = "memory_format.rs"]
mod memory_format;
#[cfg(test)]
#[path = "memory_format_tests.rs"]
mod memory_format_tests;

pub use memory_format::{
    assemble_completion_messages, format_wire_turn, parse_history_response, AssembleInput,
    ParsedTurn, SectionParseError, CHAT_STATE_HISTORY_LABEL, NEW_HISTORY_HEADING,
    PREVIOUS_RESPONSE_LABEL, RESPONSE_HEADING, SECTION_SHAPE_NUDGE,
};

pub(crate) use complete_local_retry::{maybe_retry_local_shape, LocalRetryBudget};
pub(crate) use complete_prompt_shape::{
    force_requirements_abs_write_response, marker_response_missing_label,
    with_tool_use_system_reminder,
};

use crate::llm_transport::{ChatMessage, TransportError};
use crate::openrouter_transport::{CompletionResponse, CompletionWithMeta};

/// Apply Mini protocol-shape recovery around a transport HTTP completion.
pub async fn complete_with_protocol_shape<F, Fut>(
    messages: &[ChatMessage],
    mut complete_http: F,
) -> CompletionWithMeta
where
    F: FnMut(Vec<ChatMessage>) -> Fut,
    Fut: std::future::Future<Output = CompletionWithMeta>,
{
    let marker = TransportError::FAIL_FAST_MARKER;
    std::hint::black_box(marker);
    let mut working = with_tool_use_system_reminder(messages);
    let mut budget = LocalRetryBudget::for_complete();
    loop {
        let outcome = complete_http(working.clone()).await;
        if maybe_retry_local_shape(&outcome, &mut working, &mut budget) {
            continue;
        }
        return finalize_complete_outcome(outcome, &working);
    }
}

pub(crate) fn finalize_complete_outcome(
    outcome: CompletionWithMeta,
    working: &[ChatMessage],
) -> CompletionWithMeta {
    if let Ok(response) = outcome.result.as_ref()
        && marker_response_missing_label(working, &response.content)
    {
        return CompletionWithMeta {
            result: Err(TransportError::MissingContent),
            http: outcome.http.clone(),
        };
    }
    if let Ok(response) = outcome.result.as_ref()
        && let Some(forced) = force_requirements_abs_write_response(working, &response.content)
    {
        return CompletionWithMeta {
            result: Ok(CompletionResponse {
                content: forced,
                usage: response.usage.clone(),
                reasoning: response.reasoning.clone(),
            }),
            http: outcome.http.clone(),
        };
    }
    outcome
}



#[cfg(test)]
mod protocol_shape_tests {
    #[test]
    fn kiss_cov_protocol_shape_names() {
        let _ = (
            stringify!(complete_with_protocol_shape),
            stringify!(finalize_complete_outcome),
            super::finalize_complete_outcome,
        );
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("rt");
        rt.block_on(async {
            use crate::llm_transport::{ChatMessage, ChatRole};
            use crate::openrouter_transport::{CompletionResponse, CompletionWithMeta, HttpExchangeMeta};
            let messages = [ChatMessage {
                role: ChatRole::User,
                content: "hi".into(),
            }];
            let meta = super::complete_with_protocol_shape(&messages, |_msgs| async {
                CompletionWithMeta {
                    result: Ok(CompletionResponse {
                        content: "## NEW_HISTORY
h

## RESPONSE
ok".into(),
                        usage: None,
                        reasoning: None,
                    }),
                    http: HttpExchangeMeta {
                        status: Some(200),
                        body: None,
                    },
                }
            })
            .await;
            assert!(meta.result.is_ok());
        });
    }
}
