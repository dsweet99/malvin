use super::complete_local_retry::{maybe_retry_local_shape, LocalRetryBudget};
use crate::malvin_mini::{
    format_wire_turn, ChatMessage, ChatRole, CompletionResponse, CompletionWithMeta,
    HttpExchangeMeta,
};

#[test]
fn fail_epoch_forces_act_after_nonzero_exit_without_fence() {
    let mut working = vec![ChatMessage {
        role: ChatRole::User,
        content: "Exit code 1\nstdout:\nFAILED\nstderr:\n".into(),
    }];
    let outcome = CompletionWithMeta {
        result: Ok(CompletionResponse {
            content: format_wire_turn("- note", "The live probe is outdated; CONTINUE_ROUTER"),
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
        max_marker: 1,
        max_fail_epoch: 2,
        max_transport_stall: 2,
        max_section_shape: 4,
        max_requirements_schema: 3,
    };
    assert!(maybe_retry_local_shape(&outcome, &mut working, &mut budget));
    assert_eq!(budget.fail_epoch_passes, 1);
    assert!(working[0].content.contains("Null Study under a failed live probe"));
    assert!(maybe_retry_local_shape(&outcome, &mut working, &mut budget));
    assert_eq!(budget.fail_epoch_passes, 2);
    assert!(working.iter().any(|m| m.content.contains("Emit an Act fence now")));
    assert!(maybe_retry_local_shape(&outcome, &mut working, &mut budget));
    assert_eq!(budget.missing_shape_passes, 1);
    assert!(working.iter().any(|m| {
        matches!(m.role, ChatRole::System) && m.content.contains("Thought-only responses")
    }));
    assert!(!maybe_retry_local_shape(&outcome, &mut working, &mut budget));
}

#[test]
fn exterior_without_act_forces_retry() {
    let mut working = vec![
        ChatMessage { role: ChatRole::Assistant, content: "```bash\ncurl https://ex.com\n```".into() },
        ChatMessage { role: ChatRole::User, content: "Exit code 0\nstdout:\nok\nstderr:\n".into() },
    ];
    let outcome = CompletionWithMeta {
        result: Ok(CompletionResponse {
            content: format_wire_turn("- note", "```bash\ncurl https://ex.com/2\n```"),
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
    assert!(working[0].content.contains("exterior contact before revising"));
    assert!(maybe_retry_local_shape(&outcome, &mut working, &mut b));
    assert!(working.iter().any(|m| m.content.contains("Emit an Act fence now")));
}

#[test]
fn unpaid_silence_close_forces_act() {
    let mut working = vec![ChatMessage { role: ChatRole::User, content: "Solve.".into() }];
    let outcome = CompletionWithMeta {
        result: Ok(CompletionResponse {
            content: format_wire_turn("- note", "done"),
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
    assert!(working[0].content.contains("Freeze capital is unpaid silence"));
}

