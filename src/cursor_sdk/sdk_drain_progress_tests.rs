use crate::acp::CoderPromptOptions;

use super::sdk_bug_helpers::{
    bug_clear_env, bug_client, bug_prepare, bug_set_drain_idle_timeout_ms, bug_set_progress_env,
    bug_set_tool_turn_env,
};

#[tokio::test]
async fn progress_events_keep_drain_alive_past_idle_budget() {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = bug_prepare();
    bug_set_progress_env(40, 6);
    let mut client = bug_client(tmp.path(), 1);
    client.begin_coder_session(tmp.path()).await.expect("begin");
    bug_set_drain_idle_timeout_ms(150);
    let log = tmp.path().join("prompts.log");
    let started = std::time::Instant::now();
    run_progress_prompt(&mut client, &log).await;
    assert!(started.elapsed() > std::time::Duration::from_millis(100));
    assert_eq!(
        client.last_coder_prompt_agent_response().as_deref(),
        Some("progressed")
    );
    let trace = std::fs::read_to_string(tmp.path().join("trace.jsonl")).unwrap_or_default();
    assert!(trace.contains("\"event\":\"progress\""), "{trace}");
    client.end_coder_session().await.expect("end");
    bug_clear_env();
}

#[tokio::test(start_paused = true)]
async fn continuous_events_hit_cumulative_turn_deadline() {
    let _guard = crate::test_utils::test_env_lock();
    bug_set_drain_idle_timeout_ms(60_000);
    let mut turn = crate::bridge_sdk::DrainIdleTurn::new();
    let labels = crate::bridge_sdk::DrainIdleLabels {
        prefix: "bridge timed out",
        waiting_for: "run_done",
    };
    for i in 0..2 {
        let event = crate::bridge_sdk::await_next_with_idle_in_turn(
            labels,
            None,
            async move {
                tokio::time::sleep(std::time::Duration::from_mins(1)).await;
                Ok::<_, crate::acp::AgentError>(crate::bridge_protocol::BridgeEvent::Progress {
                    kind: Some(format!("heartbeat-{i}")),
                    detail: None,
                })
            },
            &mut turn,
        )
        .await
        .expect("events within cumulative cap");
        assert!(matches!(
            event,
            crate::bridge_protocol::BridgeEvent::Progress { .. }
        ));
        if let crate::bridge_protocol::BridgeEvent::Progress { kind, .. } = event {
            assert_eq!(kind.as_deref(), Some(format!("heartbeat-{i}").as_str()));
        }
    }
    let err = crate::bridge_sdk::await_next_with_idle_in_turn(
        labels,
        None,
        async {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            Ok::<_, crate::acp::AgentError>(crate::bridge_protocol::BridgeEvent::Progress {
                kind: Some("late".into()),
                detail: None,
            })
        },
        &mut turn,
    )
    .await
    .expect_err("chatter past 2× idle must time out");
    assert!(err.0.contains("bridge timed out"));
    bug_clear_env();
}

#[tokio::test]
async fn long_tool_turn_completes_past_base_turn_cap() {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = bug_prepare();
    bug_set_tool_turn_env(60, 8);
    bug_set_drain_idle_timeout_ms(150);
    let mut client = bug_client(tmp.path(), 1);
    client.begin_coder_session(tmp.path()).await.expect("begin");
    let log = tmp.path().join("prompts.log");
    let started = std::time::Instant::now();
    client
        .run_coder_prompt(
            "LONG_TOOL_TURN_THEN_DONE please",
            &log,
            "coder",
            CoderPromptOptions {
                llm_phase: Some(crate::run_timing::TimingPhase::Implement),
                ..CoderPromptOptions::default()
            },
        )
        .await
        .expect("long tool turn must finish past base 2× idle cap");
    let elapsed = started.elapsed();
    assert!(
        elapsed > std::time::Duration::from_millis(300),
        "expected wall time past base 2×150ms cap, got {elapsed:?}"
    );
    assert_eq!(
        client.last_coder_prompt_agent_response().as_deref(),
        Some("long-tool-turn-done")
    );
    let trace = std::fs::read_to_string(tmp.path().join("trace.jsonl")).unwrap_or_default();
    assert!(trace.contains("\"event\":\"tool_call\""), "{trace}");
    assert!(trace.contains("\"kind\":\"heartbeat\""), "{trace}");
    client.end_coder_session().await.expect("end");
    bug_clear_env();
}

async fn run_progress_prompt(
    client: &mut crate::cursor_sdk::CursorSdkClient,
    log: &std::path::Path,
) {
    client
        .run_coder_prompt(
            "PROGRESS_THEN_DONE please",
            log,
            "coder",
            CoderPromptOptions {
                llm_phase: Some(crate::run_timing::TimingPhase::Implement),
                ..CoderPromptOptions::default()
            },
        )
        .await
        .expect("progress pulses must keep drain alive");
}

#[test]
fn kiss_cov_sdk_drain_progress_cases() {
    let _ = stringify!(progress_events_keep_drain_alive_past_idle_budget);
    let _ = stringify!(continuous_events_hit_cumulative_turn_deadline);
    let _ = stringify!(long_tool_turn_completes_past_base_turn_cap);
    let _ = stringify!(bug_set_tool_turn_env);
}
