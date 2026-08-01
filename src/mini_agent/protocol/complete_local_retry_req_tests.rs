use super::complete_local_retry::{maybe_retry_local_shape, LocalRetryBudget};
use crate::openrouter_transport::{
    ChatMessage, ChatRole, CompletionResponse, CompletionWithMeta, HttpExchangeMeta,
};

#[test]
fn requirements_schema_retry_on_object_shaped_json() {
    let mut working = vec![ChatMessage {
        role: ChatRole::User,
        content: "Write only the JSON at /tmp/run/review_requirements.json. \
Do not start implementing. output nothing else of substance — Pause."
            .into(),
    }];
    let bad = "## NEW_HISTORY\nlisted\n\n## RESPONSE\n```bash\ncat > /tmp/run/review_requirements.json << 'EOF'\n\
{\"groups\":[{\"title\":\"Bug\",\"requirements\":[{\"id\":\"REQ-1\",\"description\":\"fix it\"}]}]}\n\
EOF\n```\n";
    let outcome = CompletionWithMeta {
        result: Ok(CompletionResponse {
            content: bad.into(),
            usage: None,
            reasoning: None,
        }),
        http: HttpExchangeMeta {
            status: Some(200),
            body: None,
        },
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
    assert_eq!(b.requirements_schema_passes, 1);
    assert!(working.iter().any(|m| {
        matches!(m.role, ChatRole::User) && m.content.contains("requirements entries were objects")
    }));
    // Second pass: nudge already present → no further schema retry.
    assert!(!maybe_retry_local_shape(&outcome, &mut working, &mut b));
    assert_eq!(b.requirements_schema_passes, 1);
}

#[test]
fn requirements_path_retry_on_wrong_write_target() {
    let mut working = vec![ChatMessage {
        role: ChatRole::User,
        content: "Write only the JSON at /root/.malvin_home/logs/run/review_requirements.json. \
Do not start implementing. output nothing else of substance — Pause."
            .into(),
    }];
    let bad = "## NEW_HISTORY\nlisted\n\n## RESPONSE\n```bash\n\
cat > /app/review_requirements.json << 'EOF'\n\
{\"groups\":[{\"title\":\"Bug\",\"requirements\":[\"fix wrap\"]}]}\n\
EOF\n```\n";
    let outcome = CompletionWithMeta {
        result: Ok(CompletionResponse {
            content: bad.into(),
            usage: None,
            reasoning: None,
        }),
        http: HttpExchangeMeta {
            status: Some(200),
            body: None,
        },
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
    assert_eq!(b.requirements_schema_passes, 1);
    assert!(working.iter().any(|m| {
        matches!(m.role, ChatRole::User)
            && m.content.contains("was not written via ```bash to the named absolute path")
            && m.content.contains("/root/.malvin_home/logs/run/review_requirements.json")
            && m.content.contains("```bash")
            && m.content.contains("cat >")
    }));
}

#[test]
fn requirements_path_retry_on_prose_claim_without_bash() {
    let mut working = vec![ChatMessage {
        role: ChatRole::User,
        content: "Write only the JSON at /root/.malvin_home/logs/run/review_requirements.json. \
Do not start implementing. output nothing else of substance — Pause."
            .into(),
    }];
    let bad = "## NEW_HISTORY\nlisted\n\n## RESPONSE\nDone. Wrote `review_requirements.json` \
with 2 groups. All entries validated as non-empty strings.\n";
    let outcome = CompletionWithMeta {
        result: Ok(CompletionResponse {
            content: bad.into(),
            usage: None,
            reasoning: None,
        }),
        http: HttpExchangeMeta {
            status: Some(200),
            body: None,
        },
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
    assert_eq!(b.requirements_schema_passes, 1);
    assert!(working.iter().any(|m| {
        matches!(m.role, ChatRole::User) && m.content.contains("Do not claim success in prose")
    }));
    // Must prefer path retry over burning section-shape budget on already-valid wire.
    assert_eq!(b.section_shape_passes, 0);
    // Coverage: LocalRetryBudget::for_complete defaults used by complete().
    let _ = LocalRetryBudget::for_complete();
}
