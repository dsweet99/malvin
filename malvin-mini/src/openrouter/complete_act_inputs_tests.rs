use super::*;

#[test]
fn observation_nonzero_exit_and_new_request_helpers() {
    let _ = (
        stringify!(observation_reports_nonzero_exit),
        stringify!(observation_reports_zero_exit),
        stringify!(new_request_text),
        stringify!(previous_response_text),
        stringify!(latest_observation_has_nonzero_exit),
        stringify!(latest_observation_has_zero_exit),
    );
    assert!(observation_reports_nonzero_exit("Exit code 1\nok\n"));
    assert!(!observation_reports_nonzero_exit("Exit code 0\nok\n"));
    assert!(observation_reports_zero_exit("Exit code 0\nok\n"));
    assert!(!observation_reports_zero_exit("Exit code 1\nok\n"));
    assert!(!observation_reports_zero_exit("no exits here\n"));
    let msgs = [
        ChatMessage {
            role: ChatRole::Assistant,
            content: "```bash\ncurl https://x\n```".into(),
        },
        ChatMessage {
            role: ChatRole::User,
            content: "Exit code 1\n".into(),
        },
    ];
    assert_eq!(new_request_text(&msgs), Some("Exit code 1\n"));
    assert!(latest_observation_has_nonzero_exit(&msgs));
    assert!(!latest_observation_has_zero_exit(&msgs));
    assert!(previous_response_text(&msgs).unwrap().contains("curl"));
}
