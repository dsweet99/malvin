use super::complete_act_detect::{
    response_claims_write_without_fence, response_has_act_fence, response_section_or_raw,
    unpaid_prose_write_claim,
};
use crate::llm_transport::{ChatMessage, ChatRole};

#[test]
fn response_has_act_fence_ignores_new_history_fences() {
    let wire = "## NEW_HISTORY\n```bash\ncurl https://ex.com\n```\n\n## RESPONSE\nNo act here.\n";
    assert!(!response_has_act_fence(wire));
    assert!(!response_section_or_raw(wire).contains("curl"));
}

#[test]
fn response_has_act_fence_sees_response_fences() {
    let wire = "## NEW_HISTORY\n- note\n\n## RESPONSE\n```bash\necho hi\n```\n";
    assert!(response_has_act_fence(wire));
}

#[test]
fn response_has_act_fence_legacy_raw_when_no_sections() {
    assert!(response_has_act_fence("```bash\necho x\n```"));
    assert!(!response_has_act_fence("prose only"));
}

#[test]
fn prose_create_claim_without_fence_is_detected() {
    assert!(response_claims_write_without_fence(
        "## NEW_HISTORY\nx\n\n## RESPONSE\nI've created bin/csvcut as a Python script.\n"
    ));
    assert!(!response_claims_write_without_fence(
        "## NEW_HISTORY\nx\n\n## RESPONSE\n```bash\ncat > bin/csvcut <<'EOF'\nx\nEOF\n```\n"
    ));
}

#[test]
fn unpaid_prose_claim_after_read_observation() {
    let msgs = vec![
        ChatMessage {
            role: ChatRole::Assistant,
            content: "## RESPONSE\n```bash\ncat plan.md\n```\n".into(),
        },
        ChatMessage {
            role: ChatRole::User,
            content: "Exit code 0\nstdout:\nok\nstderr:\n".into(),
        },
    ];
    let pending =
        "## NEW_HISTORY\nx\n\n## RESPONSE\nI've created bin/csvcut. Ready for tests.\n";
    assert!(unpaid_prose_write_claim(&msgs, pending));
}

#[test]
fn paid_prose_claim_after_artifact_write_observation() {
    let msgs = vec![
        ChatMessage {
            role: ChatRole::Assistant,
            content: "## RESPONSE\n```bash\ncat > a.txt <<'EOF'\nx\nEOF\n```\n".into(),
        },
        ChatMessage {
            role: ChatRole::User,
            content: "Exit code 0\nstdout:\n\nstderr:\n".into(),
        },
    ];
    let pending = "## NEW_HISTORY\n- wrote a.txt\n\n## RESPONSE\nArtifact written; advancing.\n";
    assert!(!unpaid_prose_write_claim(&msgs, pending));
}
