use crate::malvin_mini::http_exchange::CompletionWithMeta;
use crate::malvin_mini::types::ChatMessage;
use super::memory_format::parse_history_response;
use super::complete_prompt_shape::{
    inject_requirements_path_nudge, inject_requirements_schema_nudge, is_short_form_marker_turn,
    mutate_messages_after_missing_content, requirements_path_needs_retry,
    response_has_object_shaped_requirements, session_is_plan_only, session_is_requirements_listing,
};
use super::complete_act_detect::{
    artifact_act_lacks_following_observation, history_has_any_artifact_act,
    history_has_exterior_without_artifact_act, latest_observation_has_nonzero_exit,
    response_has_act_fence, unpaid_prose_write_claim,
};
use super::complete_act_inputs::latest_observation_has_zero_exit;
use super::complete_fail_epoch::{
    inject_exterior_before_act_cue, inject_fail_epoch_act_cue, inject_probe_after_act_cue,
    inject_unpaid_silence_act_cue,
};
use super::complete_section_shape::inject_section_shape_nudge;
use super::complete_local_retry::LocalRetryBudget;

pub(crate) fn try_unpaid_zero_fence_as_missing(
    outcome: &CompletionWithMeta,
    working: &mut Vec<ChatMessage>,
    budget: &mut LocalRetryBudget,
) -> bool {
    let Ok(response) = outcome.result.as_ref() else {
        return false;
    };
    if budget.fail_epoch_passes < budget.max_fail_epoch
        || budget.missing_shape_passes >= budget.max_missing
        || is_short_form_marker_turn(working)
        || response_has_act_fence(response.content.as_str())
        || !serious_unpaid_debt(working, response.content.as_str())
    {
        return false;
    }
    if mutate_messages_after_missing_content(working) {
        budget.missing_shape_passes += 1;
        true
    } else {
        false
    }
}

pub(crate) fn serious_unpaid_debt(messages: &[ChatMessage], pending: &str) -> bool {
    (latest_observation_has_nonzero_exit(messages) && !response_has_act_fence(pending))
        || history_has_exterior_without_artifact_act(messages, Some(pending))
        || (artifact_act_lacks_following_observation(messages) && !response_has_act_fence(pending))
        || unpaid_prose_write_claim(messages, pending)
}

pub(crate) fn try_act_pressure_retry(
    outcome: &CompletionWithMeta,
    working: &mut Vec<ChatMessage>,
    budget: &mut LocalRetryBudget,
) -> bool {
    let Ok(response) = outcome.result.as_ref() else {
        return false;
    };
    let content = response.content.as_str();
    // Requirements / plan-only turns use dedicated shape nudges; Act pressure derails them.
    if session_is_requirements_listing(working) || session_is_plan_only(working) {
        return false;
    }
    // Guard early: exhausted budget, marker turn, unparseable wire, or no unpaid pressure.
    // Do not drown section-shape recovery with Act cues on unparseable wire turns.
    // Green observation + no unpaid debt: allow fence-less advance (avoid review Act thrash).
    if budget.fail_epoch_passes >= budget.max_fail_epoch
        || is_short_form_marker_turn(working)
        || parse_history_response(content).is_err()
        || (latest_observation_has_zero_exit(working) && !serious_unpaid_debt(working, content))
        || !needs_act_pressure(working, content)
    {
        return false;
    }
    if inject_act_pressure_cue(working, content) {
        budget.fail_epoch_passes += 1;
        return true;
    }
    // Cues already present under serious unpaid debt: mark pressure exhausted so
    // MissingContent-shaped recovery can run in the same maybe_retry pass.
    if serious_unpaid_debt(working, content) {
        budget.fail_epoch_passes = budget.max_fail_epoch;
    }
    false
}

pub(crate) fn try_section_shape_retry(
    outcome: &CompletionWithMeta,
    working: &mut Vec<ChatMessage>,
    budget: &mut LocalRetryBudget,
) -> bool {
    let Ok(response) = outcome.result.as_ref() else {
        return false;
    };
    if budget.section_shape_passes >= budget.max_section_shape
        || is_short_form_marker_turn(working)
        || parse_history_response(response.content.as_str()).is_ok()
    {
        return false;
    }
    if inject_section_shape_nudge(working) {
        budget.section_shape_passes += 1;
        true
    } else {
        false
    }
}

pub(crate) fn try_requirements_schema_retry(
    outcome: &CompletionWithMeta,
    working: &mut Vec<ChatMessage>,
    budget: &mut LocalRetryBudget,
) -> bool {
    let Ok(response) = outcome.result.as_ref() else {
        return false;
    };
    if budget.requirements_schema_passes >= budget.max_requirements_schema
        || is_short_form_marker_turn(working)
        || !session_is_requirements_listing(working)
        || !response_has_object_shaped_requirements(response.content.as_str())
    {
        return false;
    }
    if inject_requirements_schema_nudge(working) {
        budget.requirements_schema_passes += 1;
        true
    } else {
        false
    }
}

pub(crate) fn try_requirements_path_retry(
    outcome: &CompletionWithMeta,
    working: &mut Vec<ChatMessage>,
    budget: &mut LocalRetryBudget,
) -> bool {
    let Ok(response) = outcome.result.as_ref() else {
        return false;
    };
    if budget.requirements_schema_passes >= budget.max_requirements_schema
        || is_short_form_marker_turn(working)
        || !session_is_requirements_listing(working)
    {
        return false;
    }
    // Prefer the New-request path only — cue text may mention example paths like /app/….
    let expected = super::complete_prompt_shape::expected_path_from_messages(working);
    if let Some(path) = expected.as_deref()
        && super::complete_requirements_path::requirements_file_on_disk_is_valid(path)
    {
        return false;
    }
    if !requirements_path_needs_retry(response.content.as_str(), expected.as_deref()) {
        return false;
    }
    if inject_requirements_path_nudge(working, expected.as_deref()) {
        budget.requirements_schema_passes += 1;
        true
    } else {
        false
    }
}

fn needs_act_pressure(messages: &[ChatMessage], content: &str) -> bool {
    serious_unpaid_debt(messages, content)
        || (!history_has_any_artifact_act(messages) && !response_has_act_fence(content))
}

fn inject_act_pressure_cue(messages: &mut Vec<ChatMessage>, content: &str) -> bool {
    if latest_observation_has_nonzero_exit(messages) && !response_has_act_fence(content) {
        return inject_fail_epoch_act_cue(messages);
    }
    if history_has_exterior_without_artifact_act(messages, Some(content)) {
        return inject_exterior_before_act_cue(messages);
    }
    if artifact_act_lacks_following_observation(messages) && !response_has_act_fence(content) {
        return inject_probe_after_act_cue(messages);
    }
    if unpaid_prose_write_claim(messages, content) {
        return inject_unpaid_silence_act_cue(messages);
    }
    if !history_has_any_artifact_act(messages) && !response_has_act_fence(content) {
        return inject_unpaid_silence_act_cue(messages);
    }
    false
}
