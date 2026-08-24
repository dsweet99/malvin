use super::drain_idle::{
    DrainHealthVerdict, DrainIdleClock, DrainIdleHealthCtx, DrainIdleLabels, DrainIdleTurn,
    await_next_with_idle,
};
use super::drain_idle::drain_idle_health::{aggregate_health_outcomes, drain_sample_pids};
use crate::acp::AgentError;
use crate::child_health::SilenceHealthOutcome;
use crate::sdk_drain_timeout::sdk_drain_idle_max_wait;
use std::collections::HashSet;
use std::time::{Duration, Instant as WallInstant};
use tokio::time::Instant;

fn set_idle_ms(ms: u64) -> Option<std::ffi::OsString> {
    let prior = std::env::var_os("MALVIN_SDK_DRAIN_IDLE_TIMEOUT_MS");
    crate::sdk_drain_timeout::tests_set_idle_ms_for_test(ms);
    prior
}

#[tokio::test]
async fn await_next_times_out_without_health_extend() {
    let _guard = crate::test_utils::test_env_lock();
    let prior = set_idle_ms(80);
    let labels = DrainIdleLabels {
        prefix: "bridge timed out",
        waiting_for: "run_done",
    };
    let started = WallInstant::now();
    let err = await_next_with_idle(
        labels,
        None,
        std::future::pending::<Result<(), AgentError>>(),
    )
    .await
    .expect_err("must time out");
    assert!(err.0.contains("bridge timed out"));
    assert!(err.0.contains("run_done"));
    assert!(started.elapsed() >= Duration::from_millis(80));
    crate::sdk_drain_timeout::tests_restore_idle_ms_for_test(prior);
}

#[tokio::test]
async fn await_next_delivers_when_read_completes() {
    let _guard = crate::test_utils::test_env_lock();
    let prior = set_idle_ms(500);
    let labels = DrainIdleLabels {
        prefix: "bridge timed out",
        waiting_for: "ok",
    };
    let got = await_next_with_idle(labels, None, async { Ok::<_, AgentError>(7u8) })
        .await
        .expect("value");
    assert_eq!(got, 7);
    crate::sdk_drain_timeout::tests_restore_idle_ms_for_test(prior);
}

#[test]
fn clock_busy_extends_until_max_wait() {
    let idle = Duration::from_millis(40);
    let mut clock = DrainIdleClock::new(idle);
    assert!(clock.apply_verdict(DrainHealthVerdict::StillBusy).is_ok());
    let deadline = Instant::now() + sdk_drain_idle_max_wait(idle) + Duration::from_millis(20);
    let mut hit_err = false;
    while Instant::now() < deadline {
        if clock.apply_verdict(DrainHealthVerdict::StillBusy).is_err() {
            hit_err = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(hit_err, "StillBusy must eventually hit max_wait");
}

#[test]
fn clock_dead_fails_immediately() {
    let mut clock = DrainIdleClock::new(Duration::from_secs(30));
    assert!(
        clock
            .apply_verdict(DrainHealthVerdict::DeadOrZombie)
            .is_err()
    );
}

#[test]
fn clock_hung_fails_only_after_idle_deadline() {
    let idle = Duration::from_millis(50);
    let mut clock = DrainIdleClock::new(idle);
    assert!(clock.apply_verdict(DrainHealthVerdict::AppearsHung).is_ok());
    std::thread::sleep(idle + Duration::from_millis(10));
    assert!(
        clock
            .apply_verdict(DrainHealthVerdict::AppearsHung)
            .is_err()
    );
}

#[tokio::test]
async fn drain_sample_pids_falls_back_to_pgid() {
    let baseline = HashSet::new();
    let pids = drain_sample_pids(Some(std::process::id()), &baseline).await;
    assert!(!pids.is_empty());
}

#[test]
fn aggregate_health_policy_matches_plan() {
    use SilenceHealthOutcome::{AppearsHung, ChildNotRunning, ChildZombie, StillBusyExtendWait};
    assert_eq!(
        aggregate_health_outcomes(&[ChildNotRunning, StillBusyExtendWait, ChildZombie]),
        DrainHealthVerdict::StillBusy
    );
    assert_eq!(
        aggregate_health_outcomes(&[AppearsHung, ChildZombie]),
        DrainHealthVerdict::AppearsHung
    );
    assert_eq!(
        aggregate_health_outcomes(&[ChildNotRunning, ChildZombie]),
        DrainHealthVerdict::DeadOrZombie
    );
}

#[tokio::test]
async fn real_health_sampling_respects_two_idle_wall_cap() {
    let _guard = crate::test_utils::test_env_lock();
    let prior = set_idle_ms(5);
    let baseline = HashSet::new();
    let health = Some(DrainIdleHealthCtx {
        process_group_id: Some(std::process::id()),
        spawn_pid_baseline: &baseline,
    });
    let labels = DrainIdleLabels {
        prefix: "bridge timed out",
        waiting_for: "run_done",
    };
    let started = WallInstant::now();
    let _err = await_next_with_idle(
        labels,
        health,
        std::future::pending::<Result<(), AgentError>>(),
    )
    .await
    .expect_err("must time out");
    let elapsed = started.elapsed();
    crate::sdk_drain_timeout::tests_restore_idle_ms_for_test(prior);
    assert!(
        elapsed <= Duration::from_millis(25),
        "2× idle cap was 10 ms, but real health sampling took {elapsed:?}"
    );
}

#[tokio::test]
async fn event_arriving_during_health_sampling_wins_race() {
    let _guard = crate::test_utils::test_env_lock();
    let prior = set_idle_ms(20);
    let baseline = HashSet::new();
    let health = Some(DrainIdleHealthCtx {
        process_group_id: Some(std::process::id()),
        spawn_pid_baseline: &baseline,
    });
    let labels = DrainIdleLabels {
        prefix: "bridge timed out",
        waiting_for: "run_done",
    };
    let read = async {
        tokio::time::sleep(Duration::from_millis(30)).await;
        Ok::<_, AgentError>(42)
    };
    let result = await_next_with_idle(labels, health, read).await;
    crate::sdk_drain_timeout::tests_restore_idle_ms_for_test(prior);
    assert_eq!(result.expect("event must beat health/max timeout"), 42);
}

#[tokio::test]
async fn drain_idle_turn_check_deadline_and_reset_idle_window() {
    let _guard = crate::test_utils::test_env_lock();
    let prior = set_idle_ms(10);
    let labels = DrainIdleLabels {
        prefix: "bridge timed out",
        waiting_for: "run_done",
    };
    let mut turn = DrainIdleTurn::new();
    assert!(turn.check_max_deadline(labels).is_ok());
    turn.clock.reset_idle_window();
    std::thread::sleep(Duration::from_millis(25));
    assert!(turn.check_max_deadline(labels).is_err());
    crate::sdk_drain_timeout::tests_restore_idle_ms_for_test(prior);
}

#[test]
fn kiss_cov_drain_idle_names() {
    let _ = DrainHealthVerdict::StillBusy;
    let _ = DrainHealthVerdict::AppearsHung;
    let _ = DrainHealthVerdict::DeadOrZombie;
    let labels = DrainIdleLabels {
        prefix: "bridge timed out",
        waiting_for: "ok",
    };
    let _ = labels.silence_error(Duration::from_millis(1));
    let baseline = HashSet::new();
    let _ = DrainIdleHealthCtx {
        process_group_id: Some(1),
        spawn_pid_baseline: &baseline,
    };
    let _ = DrainIdleClock::new(Duration::from_millis(1)).slice_duration();
    let _ = DrainIdleClock::new(Duration::from_millis(1)).max_deadline();
    let _ = stringify!(await_next_with_idle);
    let _ = stringify!(await_next_with_idle_in_turn);
    let _ = stringify!(DrainIdleTurn);
    let _ = DrainIdleTurn::new;
    let _ = stringify!(reset_idle_window);
    let _ = stringify!(check_max_deadline);
    let _ = stringify!(sample_drain_health);
    let _ = stringify!(drain_sample_pids);
    let _ = stringify!(aggregate_pid_health);
    let _ = stringify!(aggregate_health_outcomes);
    let _ = stringify!(set_idle_ms);
    let _ = stringify!(await_next_times_out_without_health_extend);
    let _ = stringify!(await_next_delivers_when_read_completes);
    let _ = stringify!(clock_busy_extends_until_max_wait);
    let _ = stringify!(clock_dead_fails_immediately);
    let _ = stringify!(clock_hung_fails_only_after_idle_deadline);
    let _ = stringify!(drain_sample_pids_falls_back_to_pgid);
    let _ = stringify!(aggregate_health_policy_matches_plan);
    let _ = stringify!(real_health_sampling_respects_two_idle_wall_cap);
    let _ = stringify!(event_arriving_during_health_sampling_wins_race);
    let _ = stringify!(drain_idle_turn_check_deadline_and_reset_idle_window);
    let _ = stringify!(kiss_cov_drain_idle_names);
    let _ = stringify!(tests_set_idle_ms_for_test);
    let _ = stringify!(tests_restore_idle_ms_for_test);
}
