use super::{
    marker_response_missing_label, maybe_retry_local_shape, mutate_messages_after_marker_miss,
    with_tool_use_system_reminder, LocalRetryBudget,
};
use crate::error::OpenRouterError;
use crate::openrouter::{
    ChatMessage, ChatRole, CompletionResponse, CompletionWithMeta, HttpExchangeMeta,
};

fn meta(result: Result<CompletionResponse, OpenRouterError>) -> CompletionWithMeta {
    CompletionWithMeta {
        result,
        http: HttpExchangeMeta {
            status: Some(200),
            body: None,
        },
    }
}

#[test]
fn fail_epoch_forces_act_after_nonzero_exit_without_fence() {
    let mut working = vec![ChatMessage {
        role: ChatRole::User,
        content: "Exit code 1\nstdout:\nFAILED\nstderr:\n".into(),
    }];
    let outcome = meta(Ok(CompletionResponse {
        content: "The live probe is outdated; CONTINUE_ROUTER".into(),
        usage: None,
        reasoning: None,
    }));
    let mut budget = LocalRetryBudget {
        shrink_passes: 0,
        missing_shape_passes: 0,
        marker_miss_passes: 0,
        fail_epoch_passes: 0,
        transport_stall_passes: 0,
        max_shrink: 32,
        max_missing: 1,
        max_marker: 1,
        max_fail_epoch: 2,
        max_transport_stall: 2,
    };
    assert!(maybe_retry_local_shape(&outcome, &mut working, &mut budget));
    assert_eq!(budget.fail_epoch_passes, 1);
    assert!(working[0].content.contains("Null Study under a failed live probe"));
    // Cue already present — second pass appends a user Act nudge.
    assert!(maybe_retry_local_shape(&outcome, &mut working, &mut budget));
    assert_eq!(budget.fail_epoch_passes, 2);
    assert!(working.iter().any(|m| m.content.contains("Emit an Act fence now")));
    // Act-pressure exhausted → unpaid zero-fence consumes MissingContent-shaped budget.
    assert!(maybe_retry_local_shape(&outcome, &mut working, &mut budget));
    assert_eq!(budget.missing_shape_passes, 1);
    assert!(working.iter().any(|m| {
        matches!(m.role, ChatRole::System) && m.content.contains("Thought-only responses")
    }));
    // Missing budget exhausted — no further local mutate.
    assert!(!maybe_retry_local_shape(&outcome, &mut working, &mut budget));
}

#[test]
fn reminder_switches_to_fail_epoch_cue_after_nonzero_exit() {
    let green = vec![ChatMessage {
        role: ChatRole::User,
        content: "Exit code 0\nstdout:\nok\nstderr:\n".into(),
    }];
    let study = with_tool_use_system_reminder(&green);
    assert!(study[0].content.contains("request-named"));
    assert!(study[0].content.contains("private asserts"));
    assert!(!study[0].content.contains("nonzero exit"));

    let red = vec![ChatMessage {
        role: ChatRole::User,
        content: "Exit code 1\nstdout:\nFAILED\nstderr:\n".into(),
    }];
    let cue = with_tool_use_system_reminder(&red);
    assert!(cue[0].content.contains("nonzero exit is a failed live check"));
    assert!(cue[0].content.contains("acceptance region"));
}

#[test]
fn maybe_retry_local_shape_marker_miss_then_stops() {
    let mut working = vec![ChatMessage {
        role: ChatRole::User,
        content: "CODING_TASK: YES\n\nPause.".into(),
    }];
    let outcome = meta(Ok(CompletionResponse {
        content: "Investigation complete.".into(),
        usage: None,
        reasoning: None,
    }));
    let mut budget = LocalRetryBudget {
        shrink_passes: 0,
        missing_shape_passes: 0,
        marker_miss_passes: 0,
        fail_epoch_passes: 0,
        transport_stall_passes: 0,
        max_shrink: 32,
        max_missing: 1,
        max_marker: 3,
        max_fail_epoch: 2,
        max_transport_stall: 2,
    };
    assert!(maybe_retry_local_shape(&outcome, &mut working, &mut budget));
    assert_eq!(budget.marker_miss_passes, 1);
    assert!(matches!(working[0].role, ChatRole::System));
    assert!(working.iter().any(|m| m.content.starts_with("Output exactly one")));
    // Already collapsed to minimal; further mutation is exhausted.
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
    let outcome = meta(Err(OpenRouterError::MissingContent));
    let mut b = budget(2);
    assert!(maybe_retry_local_shape(&outcome, &mut working, &mut b));
    assert!(working[0].content.contains("Thought-only responses"));
    assert!(maybe_retry_local_shape(&outcome, &mut working, &mut b));
    assert!(!working.iter().any(|m| m.content.contains("request-named")));
}

fn budget(max_fail: u32) -> LocalRetryBudget {
    LocalRetryBudget {
        shrink_passes: 0,
        missing_shape_passes: 0,
        marker_miss_passes: 0,
        fail_epoch_passes: 0,
        transport_stall_passes: 0,
        max_shrink: 32,
        max_missing: 3,
        max_marker: 1,
        max_fail_epoch: max_fail,
        max_transport_stall: 2,
    }
}

#[test]
fn exterior_without_act_forces_retry() {
    let mut working = vec![
        ChatMessage { role: ChatRole::Assistant, content: "```bash\ncurl https://ex.com\n```".into() },
        ChatMessage { role: ChatRole::User, content: "Exit code 0\nstdout:\nok\nstderr:\n".into() },
    ];
    let outcome = meta(Ok(CompletionResponse { content: "```bash\ncurl https://ex.com/2\n```".into(), usage: None, reasoning: None }));
    let mut b = budget(4);
    assert!(maybe_retry_local_shape(&outcome, &mut working, &mut b));
    assert!(working[0].content.contains("exterior contact before revising"));
    assert!(maybe_retry_local_shape(&outcome, &mut working, &mut b));
    assert!(working.iter().any(|m| m.content.contains("Emit an Act fence now")));
}

#[test]
fn unpaid_silence_close_forces_act() {
    let mut working = vec![ChatMessage { role: ChatRole::User, content: "Solve.".into() }];
    let outcome = meta(Ok(CompletionResponse { content: "done".into(), usage: None, reasoning: None }));
    let mut b = budget(4);
    assert!(maybe_retry_local_shape(&outcome, &mut working, &mut b));
    assert!(working[0].content.contains("Freeze capital is unpaid silence"));
}

#[test]
fn reminder_switches_to_exterior_cue() {
    let msgs = vec![
        ChatMessage { role: ChatRole::Assistant, content: "```bash\ncurl https://ex.com\n```".into() },
        ChatMessage { role: ChatRole::User, content: "Exit code 0\nstdout:\nhtml\nstderr:\n".into() },
    ];
    assert!(with_tool_use_system_reminder(&msgs)[0].content.contains("Exterior contact before revising"));
}

#[test]
fn probe_after_act_forces_retry_on_close() {
    let mut working = vec![ChatMessage { role: ChatRole::Assistant, content: "```bash\ncat > a.txt <<'EOF'\nx\nEOF\n```".into() }];
    let outcome = meta(Ok(CompletionResponse { content: "done".into(), usage: None, reasoning: None }));
    let mut b = budget(4);
    assert!(maybe_retry_local_shape(&outcome, &mut working, &mut b));
    assert!(working[0].content.contains("revised without a following"));
}

#[test]
fn private_exit_close_still_forces_probe_when_request_names_probe() {
    let request = "Done when\n\n`python -m pytest -q` passes, and the hidden grader accepts.\n";
    let mut working = vec![
        ChatMessage {
            role: ChatRole::User,
            content: request.into(),
        },
        ChatMessage {
            role: ChatRole::Assistant,
            content: "```bash\ncat > meta/codata.py <<'PY'\nreturn 1\nPY\n```".into(),
        },
        ChatMessage {
            role: ChatRole::User,
            content: "Exit code 0\nstdout:\n137035999177000\nstderr:\n".into(),
        },
    ];
    let outcome = meta(Ok(CompletionResponse {
        content: "pytest -q passes (2 passed). Done.".into(),
        usage: None,
        reasoning: None,
    }));
    let mut b = budget(4);
    assert!(maybe_retry_local_shape(&outcome, &mut working, &mut b));
    assert!(working[0].content.contains("revised without a following"));
    assert!(working[0].content.contains("postdates"));
}

