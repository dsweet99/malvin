use super::{
    classify_turn, exhausted_error, push_user_prompt, TurnAction, LoopDriverSession,
};
use malvin_mini::ChatRole;

#[test]
fn classify_turn_detects_mini_done_and_no_fence_nudge() {
    assert!(matches!(
        classify_turn("line\nMINI_DONE\n", false),
        TurnAction::Done(_)
    ));
    assert!(matches!(
        classify_turn("no fence", false),
        TurnAction::Continue
    ));
    assert!(matches!(
        classify_turn("no fence", true),
        TurnAction::Done(_)
    ));
    assert!(matches!(
        classify_turn("```bash\necho hi\n```", false),
        TurnAction::RunBash(_)
    ));
}

#[test]
fn push_user_prompt_prepends_constraints() {
    let mut session = LoopDriverSession {
        messages: vec![],
        cwd: std::env::temp_dir(),
    };
    push_user_prompt(&mut session, "constraints", "task");
    let user = session.messages.first().expect("user");
    assert!(user.content.contains("constraints"));
    assert!(user.content.contains("task"));
    assert!(matches!(user.role, ChatRole::User));
}

#[test]
fn exhausted_error_includes_transcript() {
    let err = exhausted_error(2, "partial");
    assert!(err.0.contains("exhausted"));
    assert!(err.0.contains("partial"));
}
