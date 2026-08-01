use super::memory_format::*;
use crate::malvin_mini::types::ChatRole;

#[test]
fn parse_requires_history_before_response() {
    let ok = parse_history_response("## NEW_HISTORY\nh\n\n## RESPONSE\nr").unwrap();
    assert_eq!(ok.new_history, "h");
    assert_eq!(ok.response, "r");
    assert!(matches!(
        parse_history_response("## RESPONSE\nr\n## NEW_HISTORY\nh"),
        Err(SectionParseError::WrongOrder)
    ));
    assert!(matches!(
        parse_history_response("just prose"),
        Err(SectionParseError::MissingNewHistory)
    ));
}

#[test]
fn assemble_omits_empty_history_and_previous() {
    let msgs = assemble_completion_messages(AssembleInput {
        header: "HDR",
        study_act_cue: Some("CUE"),
        history: "",
        previous_response: "",
        new_request: "go",
    });
    assert_eq!(msgs.len(), 3);
    assert!(matches!(msgs[0].role, ChatRole::System));
    assert_eq!(msgs[0].content, "HDR");
    assert_eq!(msgs[1].content, "CUE");
    assert!(matches!(msgs[2].role, ChatRole::User));
    assert_eq!(msgs[2].content, "go");
}

#[test]
fn assemble_includes_history_and_previous() {
    let msgs = assemble_completion_messages(AssembleInput {
        header: "HDR",
        study_act_cue: None,
        history: "obj: ship",
        previous_response: "```bash\necho x\n```",
        new_request: "Exit code 0\n",
    });
    assert!(msgs
        .iter()
        .any(|m| m.content.contains(CHAT_STATE_HISTORY_LABEL)));
    assert!(msgs.iter().any(|m| matches!(m.role, ChatRole::Assistant)));
    assert_eq!(msgs.last().unwrap().content, "Exit code 0\n");
}

#[test]
fn parse_error_display_and_missing_response() {
    assert!(parse_history_response("## NEW_HISTORY\nonly")
        .unwrap_err()
        .as_message()
        .contains("RESPONSE"));
    assert!(parse_history_response("## RESPONSE\nonly")
        .unwrap_err()
        .as_message()
        .contains("NEW_HISTORY"));
    let _ = ParsedTurn {
        new_history: "a".into(),
        response: "b".into(),
    };
    let _ = (
        section_parse_error_message(&SectionParseError::MissingNewHistory),
        section_parse_error_message(&SectionParseError::MissingResponse),
        section_parse_error_message(&SectionParseError::WrongOrder),
        stringify!(as_message),
        stringify!(section_parse_error_message),
    );
}

#[test]
fn format_wire_turn_round_trips() {
    let wire = format_wire_turn("hist", "resp");
    let parsed = parse_history_response(&wire).unwrap();
    assert_eq!(parsed.new_history, "hist");
    assert_eq!(parsed.response, "resp");
}
