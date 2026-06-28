use super::{
    classify_turn, exhausted_error, push_user_prompt, TurnAction, LoopDriverConfig,
    LoopDriverSession,
};
use crate::agent_backend::mini::terminal::MiniTerminalReason;
use crate::agent_backend::test_support::loop_driver_config;
use malvin_mini::ChatRole;

#[test]
fn classify_turn_detects_mini_done_and_fenceless_completion() {
    let config = loop_driver_config(8, 1);
    let investigate = LoopDriverConfig {
        expects_investigation: true,
        ..loop_driver_config(8, 1)
    };
    assert!(matches!(
        classify_turn("line\nMINI_DONE\n", &config, false).0,
        TurnAction::Done(MiniTerminalReason::MiniDoneOutsideFence)
    ));
    assert!(matches!(
        classify_turn("no fence", &config, false).0,
        TurnAction::Done(MiniTerminalReason::FencelessComplete)
    ));
    assert!(matches!(
        classify_turn("summary after bash", &investigate, true).0,
        TurnAction::Done(MiniTerminalReason::FencelessPremature)
    ));
    assert!(matches!(
        classify_turn("```bash\necho hi\n```", &config, false).0,
        TurnAction::RunBash(_)
    ));
    assert!(matches!(
        classify_turn("```bash\nMINI_DONE\necho hi\n```", &config, false).0,
        TurnAction::RunBash(_)
    ));
}

#[test]
fn push_user_prompt_prepends_constraints() {
    let mut session = LoopDriverSession {
        messages: vec![],
        cwd: std::env::temp_dir(),
        constraints_prepended: false,
        bash_commands_this_prompt: vec![],
        prompt_index: 0,
    llm_model_slug: String::new(),
    };
    let config = loop_driver_config(8, 1);
    push_user_prompt(&mut session, &config, "task");
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
