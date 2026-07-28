use crate::error::OpenRouterError;
use crate::openrouter::http_exchange::CompletionWithMeta;
use crate::openrouter::types::ChatMessage;
use super::complete_prompt_shape::{
    is_short_form_marker_turn, marker_response_missing_label, mutate_messages_after_marker_miss,
    mutate_messages_after_missing_content,
};
use super::complete_prompt_shrink::shrink_prompt_messages;

#[path = "complete_local_retry_pressure.rs"]
mod complete_local_retry_pressure;
use complete_local_retry_pressure::{
    try_act_pressure_retry, try_requirements_path_retry, try_requirements_schema_retry,
    try_section_shape_retry, try_unpaid_zero_fence_as_missing,
};

pub(crate) struct LocalRetryBudget {
    pub shrink_passes: u32,
    pub missing_shape_passes: u32,
    pub marker_miss_passes: u32,
    pub fail_epoch_passes: u32,
    pub transport_stall_passes: u32,
    pub section_shape_passes: u32,
    pub requirements_schema_passes: u32,
    pub max_shrink: u32,
    pub max_missing: u32,
    pub max_marker: u32,
    pub max_fail_epoch: u32,
    pub max_transport_stall: u32,
    pub max_section_shape: u32,
    pub max_requirements_schema: u32,
}

impl LocalRetryBudget {
    pub(crate) fn for_complete() -> Self {
        Self {
            shrink_passes: 0,
            missing_shape_passes: 0,
            marker_miss_passes: 0,
            fail_epoch_passes: 0,
            transport_stall_passes: 0,
            section_shape_passes: 0,
            requirements_schema_passes: 0,
            max_shrink: 32,
            // Thought-only / empty-content stalls need more than one shape mutate
            // (progress cue → strip reminder → shrink) before surfacing MissingContent.
            max_missing: 4,
            max_marker: 8,
            max_fail_epoch: 4,
            max_transport_stall: 3,
            max_section_shape: 4,
            max_requirements_schema: 8,
        }
    }
}

pub(crate) fn maybe_retry_local_shape(
    outcome: &CompletionWithMeta,
    working: &mut Vec<ChatMessage>,
    budget: &mut LocalRetryBudget,
) -> bool {
    // Requirements schema/path before section-shape: listing turns often return
    // fence-less prose claims that would otherwise burn section-shape budget first.
    try_shrink_on_overflow(outcome, working, budget)
        || try_mutate_on_missing(outcome, working, budget)
        || try_mutate_on_transport_stall(outcome, working, budget)
        || try_mutate_on_marker_miss(outcome, working, budget)
        || try_requirements_schema_retry(outcome, working, budget)
        || try_requirements_path_retry(outcome, working, budget)
        || try_section_shape_retry(outcome, working, budget)
        || try_act_pressure_retry(outcome, working, budget)
        || try_unpaid_zero_fence_as_missing(outcome, working, budget)
}

fn try_shrink_on_overflow(
    outcome: &CompletionWithMeta,
    working: &mut Vec<ChatMessage>,
    budget: &mut LocalRetryBudget,
) -> bool {
    let overflow = outcome
        .result
        .as_ref()
        .err()
        .is_some_and(OpenRouterError::is_context_overflow);
    if overflow && budget.shrink_passes < budget.max_shrink && shrink_prompt_messages(working) {
        budget.shrink_passes += 1;
        true
    } else {
        false
    }
}

fn try_mutate_on_missing(
    outcome: &CompletionWithMeta,
    working: &mut Vec<ChatMessage>,
    budget: &mut LocalRetryBudget,
) -> bool {
    let missing = outcome
        .result
        .as_ref()
        .err()
        .is_some_and(|e| matches!(e, OpenRouterError::MissingContent));
    if missing
        && budget.missing_shape_passes < budget.max_missing
        && mutate_messages_after_missing_content(working)
    {
        budget.missing_shape_passes += 1;
        true
    } else {
        false
    }
}

/// Request timeouts / transport stalls often mean thought-only generation burned the
/// wall-clock. Nudge toward an Act and retry locally before surfacing to the gate.
fn try_mutate_on_transport_stall(
    outcome: &CompletionWithMeta,
    working: &mut Vec<ChatMessage>,
    budget: &mut LocalRetryBudget,
) -> bool {
    let stalled = outcome.result.as_ref().err().is_some_and(|e| {
        matches!(e, OpenRouterError::Transport(_))
            || matches!(e, OpenRouterError::ProviderTransport { .. })
    });
    if stalled
        && budget.transport_stall_passes < budget.max_transport_stall
        && !is_short_form_marker_turn(working)
        && mutate_messages_after_missing_content(working)
    {
        budget.transport_stall_passes += 1;
        true
    } else {
        false
    }
}

fn try_mutate_on_marker_miss(
    outcome: &CompletionWithMeta,
    working: &mut Vec<ChatMessage>,
    budget: &mut LocalRetryBudget,
) -> bool {
    let Ok(response) = outcome.result.as_ref() else {
        return false;
    };
    if budget.marker_miss_passes < budget.max_marker
        && marker_response_missing_label(working, &response.content)
        && mutate_messages_after_marker_miss(working)
    {
        budget.marker_miss_passes += 1;
        true
    } else {
        false
    }
}

#[cfg(test)]
#[path = "complete_local_retry_tests.rs"]
mod complete_local_retry_tests;

#[cfg(test)]
#[path = "complete_local_retry_act_tests.rs"]
mod complete_local_retry_act_tests;
#[cfg(test)]
#[path = "complete_local_retry_act_pressure_tests.rs"]
mod complete_local_retry_act_pressure_tests;

#[cfg(test)]
#[path = "complete_local_retry_req_tests.rs"]
mod complete_local_retry_req_tests;
