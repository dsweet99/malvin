use super::{
    run_inner_loop, LoopDriverConfig, LoopDriverRun, LoopDriverSession, MockStep,
};
use crate::agent_backend::test_support::{mini_test_trace, mock_llm};
use malvin_mini::CompletionResponse;

#[tokio::test]
async fn loop_driver_no_fence_triggers_nudge_before_final() {
    let llm = mock_llm(vec![
        MockStep::Ok(CompletionResponse {
            content: "no fence".into(),
            usage: None,
        }),
        MockStep::Ok(CompletionResponse {
            content: "still no".into(),
            usage: None,
        }),
    ]);
    let mut session = LoopDriverSession {
        messages: vec![],
        cwd: std::env::temp_dir(),
    };
    match run_inner_loop(LoopDriverRun {
        llm: &llm,
        session: &mut session,
        user_prompt: "go",
        config: &LoopDriverConfig {
            max_bash_turns: 2,
            max_http_retries: 3,
            mini_constraints: "constraints",
        },
        trace: &mini_test_trace(),
        timing: None,
        llm_phase: None,
        single_attempt: true,
    })
    .await
    {
        Ok(_) => panic!("fenceless completion without bash must not succeed"),
        Err(err) => assert!(err.0.contains("exhausted")),
    }
    assert!(session
        .messages
        .iter()
        .any(|m| m.content == "your last response had no ```bash``` block"));
}

#[tokio::test]
async fn loop_driver_fenceless_after_nudge_without_bash_errors() {
    let llm = mock_llm(vec![
        MockStep::Ok(CompletionResponse {
            content: "prose only".into(),
            usage: None,
        }),
        MockStep::Ok(CompletionResponse {
            content: "still prose only".into(),
            usage: None,
        }),
    ]);
    let mut session = LoopDriverSession {
        messages: vec![],
        cwd: std::env::temp_dir(),
    };
    match run_inner_loop(LoopDriverRun {
        llm: &llm,
        session: &mut session,
        user_prompt: "go",
        config: &LoopDriverConfig {
            max_bash_turns: 3,
            max_http_retries: 1,
            mini_constraints: "constraints",
        },
        trace: &mini_test_trace(),
        timing: None,
        llm_phase: None,
        single_attempt: true,
    })
    .await
    {
        Ok(_) => panic!("must not accept fenceless completion when no bash ran"),
        Err(err) => assert!(err.0.contains("exhausted")),
    }
}

#[cfg(test)]
mod kiss_cov_gate_refs {
    use super::*;

    #[test]
    fn kiss_cov_no_fence_test_symbols() {
        let _ = (
            loop_driver_no_fence_triggers_nudge_before_final,
            loop_driver_fenceless_after_nudge_without_bash_errors,
        );
    }
}
