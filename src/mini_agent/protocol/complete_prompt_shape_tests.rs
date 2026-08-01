use super::complete_prompt_shape::*;
use crate::llm_transport::{ChatMessage, ChatRole};

#[test]
fn reminder_switches_to_fail_epoch_cue_after_nonzero_exit() {
    let green = vec![ChatMessage {
        role: ChatRole::User,
        content: "Exit code 0\nstdout:\nok\nstderr:\n".into(),
    }];
    let study = with_tool_use_system_reminder(&green);
    assert!(study[0].content.contains("latest live observation exited 0"));
    assert!(!study[0].content.contains("Emit an Act"));
    assert!(!study[0].content.contains("nonzero exit"));

    let red = vec![ChatMessage {
        role: ChatRole::User,
        content: "Exit code 1\nstdout:\nFAILED\nstderr:\n".into(),
    }];
    let cue = with_tool_use_system_reminder(&red);
    assert!(cue[0].content.contains("nonzero exit is a failed live check"));
    assert!(cue[0].content.contains("acceptance region"));
    assert!(cue[0].content.contains("identical argv"));
    assert!(cue[0].content.contains("does not clear the fail epoch"));
}

#[test]
fn green_observation_requires_named_checks_still_paid() {
    let green = vec![ChatMessage {
        role: ChatRole::User,
        content: "Exit code 0\nstdout:\nok\nstderr:\n".into(),
    }];
    let study = with_tool_use_system_reminder(&green);
    assert!(study[0].content.contains("latest live observation exited 0"));
    assert!(study[0].content.contains("exact invocation"));
    assert!(study[0].content.contains("still unpaid"));
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
fn requirements_only_new_request_gets_string_schema_cue() {
    let msgs = vec![ChatMessage {
        role: ChatRole::User,
        content: "Write only the JSON at /tmp/run/review_requirements.json. \
Do not start implementing. After writing that file, output nothing else of substance — Pause."
            .into(),
    }];
    let cued = with_tool_use_system_reminder(&msgs);
    assert!(cued[0].content.contains("requirements-listing only"));
    assert!(cued[0].content.contains("plain string"));
    assert!(cued[0].content.contains("isinstance(req, str)"));
    assert!(!cued[0].content.contains("Look hard for unmet evidence first"));
}

#[test]
fn requirements_only_beats_exterior_cue() {
    let msgs = vec![
        ChatMessage {
            role: ChatRole::Assistant,
            content: "```bash\nls\n```".into(),
        },
        ChatMessage {
            role: ChatRole::User,
            content: "Write only the JSON at /tmp/run/review_requirements.json. \
Do not start implementing. Exit code 0\nstdout:\nok\nstderr:\n"
                .into(),
        },
    ];
    let cued = with_tool_use_system_reminder(&msgs);
    assert!(cued[0].content.contains("requirements-listing only"));
    assert!(!cued[0].content.contains("Exterior contact before revising"));
}

#[test]
fn object_shaped_requirements_detector() {
    assert!(response_has_object_shaped_requirements(
        r#"{"groups":[{"requirements":[{"id":"REQ-1","description":"x"}]}]}"#
    ));
    assert!(response_has_object_shaped_requirements(
        "```bash\ncat > review_requirements.json << 'EOF'\n\
{\"groups\":[{\"title\":\"t\",\"requirements\":[{\"id\":\"1\",\"description\":\"d\"}]}]}\nEOF\n```"
    ));
    assert!(!response_has_object_shaped_requirements(
        r#"{"groups":[{"title":"t","requirements":["plain string only"]}]}"#
    ));
}

#[test]
fn expected_path_and_wrong_path_helpers() {
    let req = "Write only the JSON at \
`/root/.malvin_home/logs/run/review_requirements.json`. Do not start implementing.";
    let expected = expected_review_requirements_path(req).expect("path");
    assert_eq!(
        expected,
        "/root/.malvin_home/logs/run/review_requirements.json"
    );
    assert!(requirements_path_needs_retry(
        "```bash\ncat > /app/review_requirements.json << 'EOF'\n{\"groups\":[]}\nEOF\n```",
        Some(expected)
    ));
    assert!(!requirements_path_needs_retry(
        "```bash\ncat > /root/.malvin_home/logs/run/review_requirements.json << 'EOF'\n{\"groups\":[]}\nEOF\n```",
        Some(expected)
    ));
    assert!(requirements_path_needs_retry(
        "All done. I wrote review_requirements.json with 5 groups. Pause.",
        Some(expected)
    ));
    assert!(requirements_path_needs_retry(
        "All done. I wrote review_requirements.json with 5 groups. Pause.",
        None
    ));
    assert!(!requirements_path_needs_retry(
        "```bash\ncat > /root/.malvin_home/logs/run/review_requirements.json << 'EOF'\n{\"groups\":[]}\nEOF\n```\nPause.",
        Some(expected)
    ));
}

#[test]
fn plan_only_new_request_gets_plan_cue() {
    let msgs = vec![ChatMessage {
        role: ChatRole::User,
        content: "Write the residual plan into chat. Do not edit product files in this turn. \
Do not implement yet."
            .into(),
    }];
    let cued = with_tool_use_system_reminder(&msgs);
    assert!(cued[0].content.contains("gap-analysis / residual planning only"));
    let _ = stringify!(request_text_is_plan_only);
    let _ = stringify!(study_act_cue_for);
    let _ = stringify!(select_study_act_cue);
    let _ = stringify!(new_request_is_plan_only);
}

#[test]
fn force_abs_write_when_prose_claim_lacks_bash() {
    let msgs = vec![ChatMessage {
        role: ChatRole::User,
        content: "Write **only** the JSON file at \
`/root/.malvin_home/logs/run/review_requirements.json`. \
Do **not** start implementing. output nothing else of substance — Pause."
            .into(),
    }];
    let prose = "## NEW_HISTORY\nx\n\n## RESPONSE\nDone. Wrote review_requirements.json.\n";
    let forced = force_requirements_abs_write_response(&msgs, prose).expect("force");
    assert!(forced.contains("```bash"));
    assert!(forced.contains(
        "cat > /root/.malvin_home/logs/run/review_requirements.json"
    ));
    assert!(forced.contains("\"groups\""));
    assert!(forced.contains("Plan acceptance"));
    assert!(forced.contains("documented CLI"));
    assert!(forced.contains("positional path"));
    assert!(forced.contains("stdin-only"));
    assert!(!forced.contains("Satisfy the user request as stated"));
    assert!(!requirements_path_needs_retry(
        &forced,
        Some("/root/.malvin_home/logs/run/review_requirements.json")
    ));
}

#[test]
fn path_retry_ignores_bash_sketch_in_new_history() {
    let path = "/root/.malvin_home/logs/run/review_requirements.json";
    let msgs = vec![ChatMessage {
        role: ChatRole::User,
        content: format!(
            "Write **only** the JSON file at `{path}`. \
Do **not** start implementing. output nothing else of substance — Pause."
        ),
    }];
    // Model sketched a write in NEW_HISTORY (never executed) and claimed success in RESPONSE.
    let wire = format!(
        "## NEW_HISTORY\n- planned write:\n```bash\ncat > {path} << 'EOF'\n{{\"groups\":[]}}\nEOF\n```\n\n\
## RESPONSE\nReview requirements written. One group with five requirements.\n"
    );
    assert!(
        requirements_path_needs_retry(&wire, Some(path)),
        "bash inside NEW_HISTORY must not count as an executed abs-path write"
    );
    let forced = force_requirements_abs_write_response(&msgs, &wire).expect("force");
    assert!(forced.contains(&format!("cat > {path}")));
}

#[test]
fn session_listing_detects_constraints_in_any_message() {
    let msgs = vec![
        ChatMessage {
            role: ChatRole::System,
            content: "Write **only** the JSON file at /tmp/x/review_requirements.json. \
Do **not** start implementing."
                .into(),
        },
        ChatMessage {
            role: ChatRole::User,
            content: "Exit code 0\nstdout:\nok\nstderr:\n".into(),
        },
    ];
    assert!(session_is_requirements_listing(&msgs));
}

#[test]
fn on_disk_valid_requirements_file_is_detected() {
    let path = std::env::temp_dir().join(format!(
        "malvin_req_disk_{}.json",
        std::process::id()
    ));
    std::fs::write(
        &path,
        r#"{"groups":[{"title":"t","requirements":["plain string"]}]}"#,
    )
    .expect("write");
    let path_s = path.to_string_lossy().into_owned();
    assert!(super::complete_requirements_path::requirements_file_on_disk_is_valid(
        &path_s
    ));
    // Force-write must not re-synthesize once the abs path already holds valid JSON.
    let msgs = vec![ChatMessage {
        role: ChatRole::User,
        content: format!(
            "Write **only** the JSON file at `{path_s}`. \
Do **not** start implementing. output nothing else of substance — Pause."
        ),
    }];
    let prose = "## NEW_HISTORY\nx\n\n## RESPONSE\nDone. Wrote review_requirements.json.\n";
    assert!(force_requirements_abs_write_response(&msgs, prose).is_none());
    let _ = std::fs::remove_file(&path);
}
