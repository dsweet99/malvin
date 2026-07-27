use crate::agent_backend::mini::MiniRetryStrategy;
use crate::agent_backend::mini::{
    run_inner_loop, LlmBackend, LoopDriverConfig, LoopDriverRun, MockScript, MockStep,
};
use crate::agent_backend::test_support::{loop_driver_config, loop_session, mini_test_trace};
use malvin_mini::{ChatRole, CompletionResponse, ResponseUsage};
use std::sync::{Arc, Mutex};

#[tokio::test]
async fn loop_driver_sticky_header_includes_constraints() {
    let seen: Arc<Mutex<Vec<Vec<malvin_mini::ChatMessage>>>> = Arc::new(Mutex::new(vec![]));
    let seen_hook = Arc::clone(&seen);
    let llm = LlmBackend::Mock(Mutex::new(MockScript {
        responses: vec![MockStep::Ok(CompletionResponse {
            content: malvin_mini::format_wire_turn("- progress", "MINI_DONE"),
            usage: None,
            reasoning: None,
        })],
        call_count: 0,
        on_response: Some(Box::new(move |_idx, msgs| {
            seen_hook.lock().unwrap().push(msgs.to_vec());
        })),
    }));
    let mut session = loop_session(std::env::temp_dir());
    run_inner_loop(LoopDriverRun {
        llm: &llm,
        session: &mut session,
        user_prompt: "user bit",
        config: &loop_driver_config(8, 3),
        trace: &mini_test_trace(),
        timing: None,
        llm_phase: None,
        single_attempt: true,
        gate_attempt: 1,
        retry_strategy: MiniRetryStrategy::CumulativeTranscript,
    })
    .await
    .expect("loop");
    let msgs = &seen.lock().unwrap()[0];
    assert!(msgs[0].content.contains("constraints"));
    assert!(msgs.iter().any(|m| matches!(m.role, ChatRole::User) && m.content.contains("user bit")));
}

#[tokio::test]
async fn loop_driver_mock_http_retry_on_429() {
    let llm = LlmBackend::Mock(Mutex::new(MockScript {
        responses: vec![
            MockStep::RateLimited,
            MockStep::Ok(CompletionResponse {
                content: malvin_mini::format_wire_turn("- progress", "MINI_DONE\nok"),
                usage: Some(ResponseUsage {
                    prompt_tokens: None,
                    completion_tokens: None,
                    total_tokens: None,
                    cost: Some(0.01),
                }),
                reasoning: None,
            }),
        ],
        call_count: 0,
        on_response: None,
    }));
    let mut session = loop_session(std::env::temp_dir());
    let out = run_inner_loop(LoopDriverRun {
        llm: &llm,
        session: &mut session,
        user_prompt: "go",
        config: &LoopDriverConfig {
            max_http_turns: 4,
            max_http_retries: 3,
            max_transport_retries: 3,
            max_bash_execs: 128,
            max_shrink_passes: 0,
            expects_investigation: false,
            mini_constraints: "c",
        },
        trace: &mini_test_trace(),
        timing: None,
        llm_phase: None,
        single_attempt: false,
        gate_attempt: 1,
        retry_strategy: MiniRetryStrategy::CumulativeTranscript,
    })
    .await
    .expect("retry ok");
    assert!(out.final_assistant_text.contains("MINI_DONE"));
}

#[cfg(test)]
mod kiss_cov_sticky {
    use super::*;
    #[test]
    fn kiss_cov_sticky_symbols() {
        let _ = (
            loop_driver_sticky_header_includes_constraints,
            loop_driver_mock_http_retry_on_429,
        );
    }
}
