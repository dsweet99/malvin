use super::{
    classify_turn, exhausted_error, stage_user_prompt, TurnAction, LoopDriverConfig,
    LoopDriverSession,
};
use crate::agent_backend::mini::memory_assemble::build_sticky_header;
use crate::agent_backend::mini::terminal::MiniTerminalReason;
use crate::agent_backend::test_support::loop_driver_config;
use crate::malvin_mini::ChatRole;

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
fn classify_ignores_fences_only_when_passed_response_body() {
    // NEW_HISTORY fences must not reach classify; callers pass RESPONSE only.
    let config = loop_driver_config(8, 1);
    let history_with_fence = "```bash\necho should_not_run\n```";
    assert!(matches!(
        classify_turn(history_with_fence, &config, false).0,
        TurnAction::RunBash(_)
    ));
    let response_only = "summary without fences";
    assert!(matches!(
        classify_turn(response_only, &config, false).0,
        TurnAction::Done(MiniTerminalReason::FencelessComplete)
    ));
}

#[test]
fn stage_user_prompt_sets_pending_new_request() {
    let mut session = LoopDriverSession {
        history: String::new(),
        previous_response: String::new(),
        pending_new_request: None,
        cwd: std::env::temp_dir(),
        bash_commands_this_prompt: vec![],
        prompt_index: 0,
        llm_model_slug: String::new(),
        section_shape_nudged: false,
    };
    let config = loop_driver_config(8, 1);
    stage_user_prompt(&mut session, &config, "task");
    assert_eq!(session.pending_new_request.as_deref(), Some("task"));
    let header = build_sticky_header(config.mini_constraints, "");
    assert!(header.contains("constraints"));
    assert!(header.contains("NEW_HISTORY"));
    let _ = ChatRole::User;
}

#[test]
fn exhausted_error_includes_transcript() {
    let err = exhausted_error(2, "partial");
    assert!(err.0.contains("exhausted"));
    assert!(err.0.contains("partial"));
}
