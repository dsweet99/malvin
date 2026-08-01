use super::complete_local_retry::{maybe_retry_local_shape, LocalRetryBudget};
use super::complete_prompt_shape::with_tool_use_system_reminder;
use crate::mini_agent::protocol::{
    format_wire_turn};
use crate::llm_transport::{ChatMessage, ChatRole, CompletionResponse};
use crate::openrouter_transport::{CompletionWithMeta,
    HttpExchangeMeta,
};

#[test]
fn prose_create_claim_after_read_forces_act() {
    let mut working = vec![
        ChatMessage {
            role: ChatRole::Assistant,
            content: "```bash\ncat plan.md\n```".into(),
        },
        ChatMessage {
            role: ChatRole::User,
            content: "Exit code 0\nstdout:\nImplement bin/csvcut\nstderr:\n".into(),
        },
    ];
    let outcome = CompletionWithMeta {
        result: Ok(CompletionResponse {
            content: format_wire_turn(
                "- note",
                "I've created bin/csvcut as a Python 3 stdlib script. Ready for test results.",
            ),
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

#[test]
fn probe_after_act_forces_retry_on_close() {
    let mut working = vec![ChatMessage {
        role: ChatRole::Assistant,
        content: "```bash\ncat > a.txt <<'EOF'\nx\nEOF\n```".into(),
    }];
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
    assert!(working[0].content.contains("revised without a following"));
}

#[test]
fn private_exit_close_still_forces_probe_when_request_names_probe() {
    let request = "Done when\n\n`python -m pytest -q` passes, and the hidden grader accepts.\n";
    let mut working = vec![
        ChatMessage { role: ChatRole::User, content: request.into() },
        ChatMessage {
            role: ChatRole::Assistant,
            content: "```bash\ncat > meta/codata.py <<'PY'\nreturn 1\nPY\n```".into(),
        },
        ChatMessage {
            role: ChatRole::User,
            content: "Exit code 0\nstdout:\n137035999177000\nstderr:\n".into(),
        },
    ];
    let outcome = CompletionWithMeta {
        result: Ok(CompletionResponse {
            content: format_wire_turn("- note", "pytest -q passes (2 passed). Done."),
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
    assert!(working[0].content.contains("revised without a following"));
    assert!(working[0].content.contains("postdates"));
}

#[test]
fn green_observation_skips_act_pressure_when_debt_paid() {
    let mut working = vec![
        ChatMessage {
            role: ChatRole::Assistant,
            content: "```bash\ncat > a.txt <<'EOF'\nx\nEOF\n```".into(),
        },
        ChatMessage {
            role: ChatRole::User,
            content: "Exit code 0\nstdout:\n\nstderr:\n".into(),
        },
    ];
    let outcome = CompletionWithMeta {
        result: Ok(CompletionResponse {
            content: "## NEW_HISTORY\n- wrote a.txt\n\n## RESPONSE\nArtifact written; advancing.\n".into(),
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
    assert!(!maybe_retry_local_shape(&outcome, &mut working, &mut b));
    assert_eq!(b.fail_epoch_passes, 0);
    assert_eq!(b.section_shape_passes, 0);
    let cued = with_tool_use_system_reminder(&working);
    assert!(cued[0].content.contains("latest live observation exited 0"));
}
