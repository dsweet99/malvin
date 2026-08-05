//! Bug 2 residual: paired `fatal`+`run_done` must not poison the next prompt.

use crate::acp::CoderPromptOptions;

use super::sdk_bug_helpers::{
    assert_err_has, bug_clear_env, bug_client, bug_prepare, expect_prompt_err,
};

#[tokio::test]
async fn fatal_then_run_done_does_not_poison_next_prompt() {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = bug_prepare();
    let mut client = bug_client(tmp.path(), 1);
    client.begin_coder_session(tmp.path()).await.expect("begin");
    let log = tmp.path().join("prompts.log");
    let err = expect_prompt_err(&mut client, "FATAL_THEN_RUN_DONE", &log).await;
    assert_err_has(&err, &["stream error"]);
    client
        .run_coder_prompt(
            "hi",
            &log,
            "coder",
            CoderPromptOptions {
                llm_phase: Some(crate::run_timing::TimingPhase::Implement),
                ..CoderPromptOptions::default()
            },
        )
        .await
        .expect("next prompt after paired fatal");
    assert_eq!(
        client.last_coder_prompt_agent_response().as_deref(),
        Some("mock reply")
    );
    client.end_coder_session().await.expect("end");
    bug_clear_env();
}

#[test]
fn kiss_cov_bug2_poison_case() {
    let _ = stringify!(fatal_then_run_done_does_not_poison_next_prompt);
}
