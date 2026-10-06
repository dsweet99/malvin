use crate::agent_process::CoderPromptOptions;

use super::sdk_bug_helpers::{
    bug_clear_env, bug_client, bug_prepare, bug_set_drain_idle_timeout_ms, bug_set_progress_env,
    bug_set_tool_turn_env,
};

#[tokio::test]
async fn progress_events_keep_drain_alive_past_idle_budget() {
    let _guard = crate::test_support::test_utils::test_env_lock();
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

#[tokio::test]
async fn heartbeat_only_turn_runs_past_two_idle_windows() {
    let _guard = crate::test_support::test_utils::test_env_lock();
    let tmp = bug_prepare();
    bug_set_progress_env(60, 8);
    bug_set_drain_idle_timeout_ms(150);
    let mut client = bug_client(tmp.path(), 1);
    client.begin_coder_session(tmp.path()).await.expect("begin");
    let log = tmp.path().join("prompts.log");
    let started = std::time::Instant::now();
    run_progress_prompt(&mut client, &log).await;
    let elapsed = started.elapsed();
    assert!(
        elapsed > std::time::Duration::from_millis(300),
        "expected heartbeats to carry the turn past 2×150ms, got {elapsed:?}"
    );
    assert_eq!(
        client.last_coder_prompt_agent_response().as_deref(),
        Some("progressed")
    );
    client.end_coder_session().await.expect("end");
    bug_clear_env();
}

#[tokio::test(start_paused = true)]
async fn continuous_events_have_no_cumulative_turn_deadline() {
    let _guard = crate::test_support::test_utils::test_env_lock();
    bug_set_drain_idle_timeout_ms(60_000);
    let mut turn = crate::backends::bridge_sdk::DrainIdleTurn::new();
    let labels = crate::backends::bridge_sdk::DrainIdleLabels {
        prefix: "bridge timed out",
        waiting_for: "run_done",
    };
    for i in 0..30 {
        let event = crate::backends::bridge_sdk::await_next_with_idle_in_turn(
            labels,
            None,
            async move {
                tokio::time::sleep(std::time::Duration::from_mins(1)).await;
                Ok::<_, crate::agent_process::AgentError>(crate::backends::cursor_sdk::protocol::BridgeEvent::Progress {
                    kind: Some(format!("heartbeat-{i}")),
                    detail: None,
                })
            },
            &mut turn,
        )
        .await
        .expect("events inside the idle window must keep the turn alive indefinitely");
        assert!(matches!(
            event,
            crate::backends::cursor_sdk::protocol::BridgeEvent::Progress { .. }
        ));
        if let crate::backends::cursor_sdk::protocol::BridgeEvent::Progress { kind, .. } = event {
            assert_eq!(kind.as_deref(), Some(format!("heartbeat-{i}").as_str()));
        }
    }
    bug_clear_env();
}

#[tokio::test]
async fn long_tool_turn_runs_past_two_idle_windows() {
    let _guard = crate::test_support::test_utils::test_env_lock();
    let tmp = bug_prepare();
    bug_set_tool_turn_env(60, 8);
    bug_set_drain_idle_timeout_ms(150);
    let mut client = bug_client(tmp.path(), 1);
    client.begin_coder_session(tmp.path()).await.expect("begin");
    let elapsed = run_long_tool_prompt(&mut client, &tmp.path().join("prompts.log")).await;
    assert!(
        elapsed > std::time::Duration::from_millis(300),
        "expected wall time past 2×150ms, got {elapsed:?}"
    );
    assert_long_tool_turn_done(&client, tmp.path());
    client.end_coder_session().await.expect("end");
    bug_clear_env();
}

async fn run_long_tool_prompt(
    client: &mut crate::backends::cursor_sdk::CursorSdkClient,
    log: &std::path::Path,
) -> std::time::Duration {
    let started = std::time::Instant::now();
    client
        .active_coder_session()
        .expect("active coder session")
        .run_coder_prompt(
            "LONG_TOOL_TURN_THEN_DONE please",
            log,
            "coder",
            CoderPromptOptions {
                llm_phase: Some(crate::run_timing::TimingPhase::Implement),
                ..CoderPromptOptions::default()
            },
        )
        .await
        .expect("long tool turn must finish past 2× idle");
    started.elapsed()
}

fn assert_long_tool_turn_done(
    client: &crate::backends::cursor_sdk::CursorSdkClient,
    tmp_dir: &std::path::Path,
) {
    assert_eq!(
        client.last_coder_prompt_agent_response().as_deref(),
        Some("long-tool-turn-done")
    );
    let trace = std::fs::read_to_string(tmp_dir.join("trace.jsonl")).unwrap_or_default();
    assert!(trace.contains("\"event\":\"tool_call\""), "{trace}");
    assert!(trace.contains("\"kind\":\"heartbeat\""), "{trace}");
}

async fn run_progress_prompt(
    client: &mut crate::backends::cursor_sdk::CursorSdkClient,
    log: &std::path::Path,
) {
    client
        .active_coder_session()
        .expect("active coder session")
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
    let _ = stringify!(heartbeat_only_turn_runs_past_two_idle_windows);
    let _ = stringify!(continuous_events_have_no_cumulative_turn_deadline);
    let _ = stringify!(long_tool_turn_runs_past_two_idle_windows);
    let _ = stringify!(bug_set_tool_turn_env);
}
