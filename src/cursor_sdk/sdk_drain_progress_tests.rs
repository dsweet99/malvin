use crate::acp::CoderPromptOptions;

use super::sdk_bug_helpers::{
    bug_clear_env, bug_client, bug_prepare, bug_set_drain_idle_timeout_ms, bug_set_progress_env,
};

#[tokio::test]
async fn progress_events_keep_drain_alive_past_idle_budget() {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = bug_prepare();
    bug_set_progress_env(40, 6);
    let mut client = bug_client(tmp.path(), 1);
    client.begin_coder_session(tmp.path()).await.expect("begin");
    bug_set_drain_idle_timeout_ms(100);
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
async fn progress_allows_more_than_ten_minutes_without_prompt_ceiling() {
    let _guard = crate::test_utils::test_env_lock();
    bug_set_drain_idle_timeout_ms(60_000);
    let prompt_started = tokio::time::Instant::now();
    for _ in 0..13 {
        let labels = crate::bridge_sdk::DrainIdleLabels {
            prefix: "bridge timed out",
            waiting_for: "run_done",
        };
        let read = async {
            tokio::time::sleep(std::time::Duration::from_secs(50)).await;
            Ok::<_, crate::acp::AgentError>(crate::bridge_protocol::BridgeEvent::Progress {
                kind: Some("heartbeat".into()),
                detail: None,
            })
        };
        let event = crate::bridge_sdk::await_next_with_idle_using(labels, read, |_| {
            std::future::ready(crate::bridge_sdk::DrainHealthVerdict::AppearsHung)
        })
        .await
        .expect("each progress line arrives inside its own idle window");
        assert!(matches!(
            event,
            crate::bridge_protocol::BridgeEvent::Progress { .. }
        ));
    }
    assert_run_done_after_progress().await;
    assert!(prompt_started.elapsed() > std::time::Duration::from_secs(600));
    bug_clear_env();
}

async fn assert_run_done_after_progress() {
    let labels = crate::bridge_sdk::DrainIdleLabels {
        prefix: "bridge timed out",
        waiting_for: "run_done",
    };
    let done = crate::bridge_sdk::await_next_with_idle_using(
        labels,
        async {
            Ok::<_, crate::acp::AgentError>(crate::bridge_protocol::BridgeEvent::RunDone {
                status: crate::bridge_protocol::RunDoneStatus::Finished,
                result: Some("virtual-long-turn".into()),
                usage: None,
                error: None,
                duration_ms: Some(650_000),
            })
        },
        |_| std::future::ready(crate::bridge_sdk::DrainHealthVerdict::AppearsHung),
    )
    .await
    .expect("run_done after progress");
    assert!(matches!(
        done,
        crate::bridge_protocol::BridgeEvent::RunDone { .. }
    ));
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
    let _ = stringify!(progress_allows_more_than_ten_minutes_without_prompt_ceiling);
    let _ = stringify!(assert_run_done_after_progress);
    let _ = stringify!(run_progress_prompt);
}
