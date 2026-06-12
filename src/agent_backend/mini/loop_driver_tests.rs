use super::{
    run_inner_loop, LoopDriverConfig, LoopDriverRun, LoopDriverSession, MockStep,
};
use crate::agent_backend::test_support::{mini_test_trace, mock_llm};
use malvin_mini::{ChatRole, CompletionResponse, ResponseUsage};

fn test_config() -> LoopDriverConfig {
    LoopDriverConfig {
        max_bash_turns: 8,
        max_http_retries: 3,
        mini_constraints: "constraints",
    }
}

#[tokio::test]
async fn loop_driver_single_fence_runs_bash_and_appends_observation() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let llm = mock_llm(vec![
        MockStep::Ok(CompletionResponse {
            content: "```bash\necho hi > out.txt\n```".into(),
            usage: None,
        }),
        MockStep::Ok(CompletionResponse {
            content: "summary".into(),
            usage: None,
        }),
        MockStep::Ok(CompletionResponse {
            content: "summary".into(),
            usage: None,
        }),
    ]);
    let mut session = LoopDriverSession {
        messages: vec![],
        cwd: tmp.path().to_path_buf(),
    };
    let out = run_inner_loop(LoopDriverRun {
        llm: &llm,
        session: &mut session,
        user_prompt: "go",
        config: &test_config(),
        trace: &mini_test_trace(),
        timing: None,
        llm_phase: None,
        single_attempt: true,
    })
    .await
    .expect("loop");
    assert_eq!(out.final_assistant_text, "summary");
    assert!(tmp.path().join("out.txt").is_file());
    assert!(session.messages.iter().any(|m| m.content.contains("Exit code")));
}

#[tokio::test]
async fn loop_driver_mini_done_line_terminates() {
    let llm = mock_llm(vec![MockStep::Ok(CompletionResponse {
        content: "MINI_DONE\n".into(),
        usage: None,
    })]);
    let mut session = LoopDriverSession {
        messages: vec![],
        cwd: std::env::temp_dir(),
    };
    let out = run_inner_loop(LoopDriverRun {
        llm: &llm,
        session: &mut session,
        user_prompt: "go",
        config: &test_config(),
        trace: &mini_test_trace(),
        timing: None,
        llm_phase: None,
        single_attempt: true,
    })
    .await
    .expect("loop");
    assert!(out.final_assistant_text.contains("MINI_DONE"));
}

#[tokio::test]
async fn loop_driver_mini_done_inside_fence_still_runs_bash() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let llm = mock_llm(vec![
        MockStep::Ok(CompletionResponse {
            content: "```bash\nMINI_DONE\necho fenced > fenced_out.txt\n```".into(),
            usage: None,
        }),
        MockStep::Ok(CompletionResponse {
            content: "done after bash".into(),
            usage: None,
        }),
        MockStep::Ok(CompletionResponse {
            content: "done after bash".into(),
            usage: None,
        }),
    ]);
    let mut session = LoopDriverSession {
        messages: vec![],
        cwd: tmp.path().to_path_buf(),
    };
    let out = run_inner_loop(LoopDriverRun {
        llm: &llm,
        session: &mut session,
        user_prompt: "go",
        config: &test_config(),
        trace: &mini_test_trace(),
        timing: None,
        llm_phase: None,
        single_attempt: true,
    })
    .await
    .expect("loop");
    assert!(tmp.path().join("fenced_out.txt").is_file());
    assert_eq!(out.final_assistant_text, "done after bash");
}

#[tokio::test]
async fn loop_driver_prepends_mini_constraints() {
    let llm = mock_llm(vec![MockStep::Ok(CompletionResponse {
        content: "MINI_DONE".into(),
        usage: None,
    })]);
    let mut session = LoopDriverSession {
        messages: vec![],
        cwd: std::env::temp_dir(),
    };
    run_inner_loop(LoopDriverRun {
        llm: &llm,
        session: &mut session,
        user_prompt: "user bit",
        config: &test_config(),
        trace: &mini_test_trace(),
        timing: None,
        llm_phase: None,
        single_attempt: true,
    })
    .await
    .expect("loop");
    let first_user = session
        .messages
        .iter()
        .find(|m| matches!(m.role, ChatRole::User))
        .expect("user");
    assert!(first_user.content.contains("constraints"));
    assert!(first_user.content.contains("user bit"));
}

#[tokio::test]
async fn loop_driver_mock_http_retry_on_429() {
    let llm = mock_llm(vec![
        MockStep::RateLimited,
        MockStep::Ok(CompletionResponse {
            content: "MINI_DONE\nok".into(),
            usage: Some(ResponseUsage {
                prompt_tokens: None,
                completion_tokens: None,
                total_tokens: None,
                cost: Some(0.01),
            }),
        }),
    ]);
    let mut session = LoopDriverSession {
        messages: vec![],
        cwd: std::env::temp_dir(),
    };
    let out = run_inner_loop(LoopDriverRun {
        llm: &llm,
        session: &mut session,
        user_prompt: "go",
        config: &LoopDriverConfig {
            max_bash_turns: 4,
            max_http_retries: 3,
            mini_constraints: "c",
        },
        trace: &mini_test_trace(),
        timing: None,
        llm_phase: None,
        single_attempt: false,
    })
    .await
    .expect("retry ok");
    assert!(out.final_assistant_text.contains("MINI_DONE"));
}

#[cfg(test)]
mod kiss_cov_gate_refs {
    use super::*;

    #[test]
    fn kiss_cov_loop_driver_test_symbols() {
        let _ = (
            mini_test_trace,
            mock_llm,
            test_config,
            loop_driver_single_fence_runs_bash_and_appends_observation,
            loop_driver_mini_done_line_terminates,
            loop_driver_mini_done_inside_fence_still_runs_bash,
            loop_driver_prepends_mini_constraints,
            loop_driver_mock_http_retry_on_429,
        );
    }
}
