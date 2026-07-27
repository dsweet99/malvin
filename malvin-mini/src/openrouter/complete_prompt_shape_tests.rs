use super::with_tool_use_system_reminder;
use crate::openrouter::{ChatMessage, ChatRole};

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
    assert!(!cued[0].content.contains("Look hard for unmet evidence first"));
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
}
