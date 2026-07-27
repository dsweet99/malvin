use super::{maybe_retry_local_shape, LocalRetryBudget};
use super::super::complete_prompt_shape::{
    marker_response_missing_label, mutate_messages_after_marker_miss,
};
use crate::error::OpenRouterError;
use crate::openrouter::{
    ChatMessage, ChatRole, CompletionResponse, CompletionWithMeta, HttpExchangeMeta,
};

#[test]
fn maybe_retry_local_shape_marker_miss_then_stops() {
    let mut working = vec![ChatMessage {
        role: ChatRole::User,
        content: "CODING_TASK: YES\n\nPause.".into(),
    }];
    let outcome = CompletionWithMeta {
        result: Ok(CompletionResponse {
            content: "Investigation complete.".into(),
            usage: None,
            reasoning: None,
        }),
        http: HttpExchangeMeta { status: Some(200), body: None },
    };
    let mut budget = LocalRetryBudget {
        shrink_passes: 0,
        missing_shape_passes: 0,
        marker_miss_passes: 0,
        fail_epoch_passes: 0,
        transport_stall_passes: 0,
        section_shape_passes: 0,
        requirements_schema_passes: 0,
        max_shrink: 32,
        max_missing: 1,
        max_marker: 3,
        max_fail_epoch: 2,
        max_transport_stall: 2,
        max_section_shape: 4,
        max_requirements_schema: 3,
    };
    assert!(maybe_retry_local_shape(&outcome, &mut working, &mut budget));
    assert_eq!(budget.marker_miss_passes, 1);
    assert!(matches!(working[0].role, ChatRole::System));
    assert!(working.iter().any(|m| m.content.starts_with("Output exactly one")));
    assert!(!maybe_retry_local_shape(&outcome, &mut working, &mut budget));
    assert_eq!(budget.marker_miss_passes, 1);
    assert!(marker_response_missing_label(
        &[ChatMessage {
            role: ChatRole::User,
            content: "CODING_TASK: YES\n\nPause.".into(),
        }],
        "no label here"
    ));
    assert!(marker_response_missing_label(
        &[ChatMessage {
            role: ChatRole::User,
            content: "Write COMPLEXITY_SCORE: n then Pause.".into(),
        }],
        "Pause."
    ));
    let mut again = working.clone();
    assert!(!mutate_messages_after_marker_miss(&mut again));
}

#[test]
fn maybe_retry_local_shape_missing_content_mutates() {
    let mut working = vec![
        ChatMessage {
            role: ChatRole::System,
            content: "State the problem and rival readings before acting. unpaid request-named probes".into(),
        },
        ChatMessage { role: ChatRole::User, content: "hi".into() },
    ];
    let outcome = CompletionWithMeta {
        result: Err(OpenRouterError::MissingContent),
        http: HttpExchangeMeta { status: Some(200), body: None },
    };
    let mut b = LocalRetryBudget {
        shrink_passes: 0,
        missing_shape_passes: 0,
        marker_miss_passes: 0,
        fail_epoch_passes: 0,
        transport_stall_passes: 0,
        section_shape_passes: 0,
        requirements_schema_passes: 0,
        max_shrink: 32,
        max_missing: 3,
        max_marker: 1,
        max_fail_epoch: 2,
        max_transport_stall: 2,
        max_section_shape: 4,
        max_requirements_schema: 3,
    };
    assert!(maybe_retry_local_shape(&outcome, &mut working, &mut b));
    assert!(working[0].content.contains("Thought-only responses"));
    assert!(maybe_retry_local_shape(&outcome, &mut working, &mut b));
    assert!(!working.iter().any(|m| m.content.contains("request-named")));
}

#[test]
fn section_shape_retry_on_missing_new_history() {
    let mut working = vec![ChatMessage {
        role: ChatRole::User,
        content: "Fix the ring buffer.".into(),
    }];
    let outcome = CompletionWithMeta {
        result: Ok(CompletionResponse {
            content: "```bash\ncat src/ringbuf.py\n```".into(),
            usage: None,
            reasoning: None,
        }),
        http: HttpExchangeMeta { status: Some(200), body: None },
    };
    let mut b = LocalRetryBudget {
        shrink_passes: 0,
        missing_shape_passes: 0,
        marker_miss_passes: 0,
        fail_epoch_passes: 0,
        transport_stall_passes: 0,
        section_shape_passes: 0,
        requirements_schema_passes: 0,
        max_shrink: 32,
        max_missing: 3,
        max_marker: 1,
        max_fail_epoch: 4,
        max_transport_stall: 2,
        max_section_shape: 4,
        max_requirements_schema: 3,
    };
    assert!(maybe_retry_local_shape(&outcome, &mut working, &mut b));
    assert_eq!(b.section_shape_passes, 1);
    assert!(working[0].content.contains("Do not omit either heading"));
    assert!(working[0].content.contains("## NEW_HISTORY"));
    assert!(maybe_retry_local_shape(&outcome, &mut working, &mut b));
    assert_eq!(b.section_shape_passes, 2);
    assert!(working.iter().any(|m| {
        matches!(m.role, ChatRole::User) && m.content.contains("omitted the required wire sections")
    }));
    let before = b.fail_epoch_passes;
    assert!(maybe_retry_local_shape(&outcome, &mut working, &mut b));
    assert_eq!(b.section_shape_passes, 3);
    assert!(working.iter().any(|m| {
        matches!(m.role, ChatRole::User) && m.content.contains("Wire format still wrong")
    }));
    assert_eq!(b.fail_epoch_passes, before);
    assert!(maybe_retry_local_shape(&outcome, &mut working, &mut b));
    assert_eq!(b.section_shape_passes, 4);
    assert!(!maybe_retry_local_shape(&outcome, &mut working, &mut b));
    assert_eq!(b.fail_epoch_passes, before);
}
