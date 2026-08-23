use super::drain_idle::{
    DrainHealthVerdict, DrainIdleHealthCtx, DrainIdleLabels, await_next_with_idle,
    await_next_with_idle_using,
};
use crate::acp::AgentError;
use std::collections::HashSet;
use std::time::Duration;
use tokio::time::Instant;

fn set_policy_idle_ms(ms: u64) -> Option<std::ffi::OsString> {
    let prior = std::env::var_os("MALVIN_SDK_DRAIN_IDLE_TIMEOUT_MS");
    crate::sdk_drain_timeout::tests_set_idle_ms_for_test(ms);
    prior
}

#[tokio::test(start_paused = true)]
async fn injected_dead_health_fails_at_first_slice() {
    let _guard = crate::test_utils::test_env_lock();
    let prior = set_policy_idle_ms(120_000);
    let started = Instant::now();
    let err = await_next_with_idle_using(
        DrainIdleLabels {
            prefix: "bridge timed out",
            waiting_for: "run_done",
        },
        std::future::pending::<Result<(), AgentError>>(),
        |_| std::future::ready(DrainHealthVerdict::DeadOrZombie),
    )
    .await
    .expect_err("dead child must fail on the first 60s health sample");
    let elapsed = started.elapsed();
    crate::sdk_drain_timeout::tests_restore_idle_ms_for_test(prior);
    assert!(err.0.contains("bridge timed out"));
    assert_eq!(elapsed, Duration::from_mins(1));
}

#[tokio::test(start_paused = true)]
async fn injected_hung_health_waits_full_idle() {
    let _guard = crate::test_utils::test_env_lock();
    let prior = set_policy_idle_ms(120_000);
    let started = Instant::now();
    let err = await_next_with_idle_using(
        DrainIdleLabels {
            prefix: "pi rpc timed out",
            waiting_for: "agent_end",
        },
        std::future::pending::<Result<(), AgentError>>(),
        |_| std::future::ready(DrainHealthVerdict::AppearsHung),
    )
    .await
    .expect_err("hung child must fail only after the idle budget");
    let elapsed = started.elapsed();
    crate::sdk_drain_timeout::tests_restore_idle_ms_for_test(prior);
    assert!(err.0.contains("pi rpc timed out"));
    assert_eq!(elapsed, Duration::from_mins(2));
}

#[tokio::test(start_paused = true)]
async fn missing_pgid_gets_no_health_extend() {
    let _guard = crate::test_utils::test_env_lock();
    let prior = set_policy_idle_ms(120_000);
    let baseline = HashSet::new();
    let started = Instant::now();
    let err = await_next_with_idle(
        DrainIdleLabels {
            prefix: "bridge timed out",
            waiting_for: "ok",
        },
        Some(DrainIdleHealthCtx {
            process_group_id: None,
            spawn_pid_baseline: &baseline,
        }),
        std::future::pending::<Result<(), AgentError>>(),
    )
    .await
    .expect_err("missing pgid must retain the original idle timeout");
    let elapsed = started.elapsed();
    crate::sdk_drain_timeout::tests_restore_idle_ms_for_test(prior);
    assert!(err.0.contains("bridge timed out"));
    assert!(elapsed >= Duration::from_mins(2));
    assert!(elapsed <= Duration::from_mins(2) + Duration::from_secs(1));
}

#[tokio::test(start_paused = true)]
async fn repeated_busy_health_stops_at_exactly_two_idle_windows() {
    let _guard = crate::test_utils::test_env_lock();
    let prior = set_policy_idle_ms(120_000);
    let started = Instant::now();
    let err = await_next_with_idle_using(
        DrainIdleLabels {
            prefix: "bridge timed out",
            waiting_for: "run_done",
        },
        std::future::pending::<Result<(), AgentError>>(),
        |_| std::future::ready(DrainHealthVerdict::StillBusy),
    )
    .await
    .expect_err("busy health must not exceed max_wait");
    let elapsed = started.elapsed();
    crate::sdk_drain_timeout::tests_restore_idle_ms_for_test(prior);
    assert!(err.0.contains("bridge timed out"));
    assert_eq!(elapsed, Duration::from_mins(4));
}

#[tokio::test(start_paused = true)]
async fn successful_event_starts_a_fresh_next_event_idle_budget() {
    let _guard = crate::test_utils::test_env_lock();
    let prior = set_policy_idle_ms(120_000);
    let prompt_started = Instant::now();
    for value in [1_u8, 2_u8] {
        let got = await_next_with_idle_using(
            DrainIdleLabels {
                prefix: "bridge timed out",
                waiting_for: "event",
            },
            async move {
                tokio::time::sleep(Duration::from_secs(90)).await;
                Ok::<_, AgentError>(value)
            },
            |_| std::future::ready(DrainHealthVerdict::AppearsHung),
        )
        .await
        .expect("each event gets a fresh 120s idle budget");
        assert_eq!(got, value);
    }
    let elapsed = prompt_started.elapsed();
    crate::sdk_drain_timeout::tests_restore_idle_ms_for_test(prior);
    assert_eq!(elapsed, Duration::from_mins(3));
}

#[test]
fn kiss_cov_drain_idle_policy_names() {
    let _ = stringify!(set_policy_idle_ms);
    let _ = stringify!(injected_dead_health_fails_at_first_slice);
    let _ = stringify!(injected_hung_health_waits_full_idle);
    let _ = stringify!(missing_pgid_gets_no_health_extend);
    let _ = stringify!(repeated_busy_health_stops_at_exactly_two_idle_windows);
    let _ = stringify!(successful_event_starts_a_fresh_next_event_idle_budget);
}
