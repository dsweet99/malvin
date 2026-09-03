use std::time::{Duration, Instant};

use super::CursorSdkClient;
use super::client_mock_tests::{
    clear_mock_bridge_env, install_mock_bridge_env, mock_bridge_path, mock_client, prompt_once,
};
use crate::bridge_sdk::SDK_BRIDGE_MAX_AGE;

struct EnsureFixture {
    client: CursorSdkClient,
    tmp: tempfile::TempDir,
    started: Instant,
}

fn bridge_started_at(client: &CursorSdkClient) -> Instant {
    crate::agent_backend::live_session(client)
        .and_then(|s| s.as_cursor())
        .expect("open")
        .started_at
}

fn backdate_bridge(client: &mut CursorSdkClient, started: Instant) {
    crate::agent_backend::live_session_mut(client)
        .and_then(|s| s.as_cursor_mut())
        .expect("open")
        .started_at = started
        .checked_sub(SDK_BRIDGE_MAX_AGE + Duration::from_secs(1))
        .expect("backdate");
}

fn count_agent_start_log_lines(stdout_log: &std::path::Path, model: &str) -> usize {
    let text = std::fs::read_to_string(stdout_log).unwrap_or_default();
    let delim = crate::output::format_who_tag_delim(crate::output::WHO_A);
    let needle = format!("{delim}{model}");
    text.lines().filter(|line| line.contains(&needle)).count()
}

async fn open_ensure_fixture() -> EnsureFixture {
    install_mock_bridge_env(&mock_bridge_path());
    let tmp = tempfile::tempdir().expect("tmp");
    let stdout_log = tmp.path().join("stdout.log");
    crate::output::set_stdout_log_path(Some(stdout_log.clone()));
    let mut client = mock_client(tmp.path());
    let _ = client.attach_run_timing_for_session();
    let started_new = client
        .ensure_coder_session(tmp.path())
        .await
        .expect("ensure");
    assert!(
        started_new.is_fresh(),
        "first ensure must create a fresh agent context"
    );
    assert_eq!(
        count_agent_start_log_lines(&stdout_log, &client.model.canonical()),
        1,
        "fresh agent start must log a|<model>"
    );
    let started = bridge_started_at(&client);
    EnsureFixture {
        client,
        tmp,
        started,
    }
}

async fn end_fixture(mut fixture: EnsureFixture) {
    fixture.client.end_coder_session().await.expect("end");
    crate::output::set_stdout_log_path(None);
    clear_mock_bridge_env();
}

#[tokio::test]
async fn cursor_sdk_ensure_reuses_fresh_bridge() {
    let _guard = crate::test_utils::test_env_lock();
    let mut fixture = open_ensure_fixture().await;
    let stdout_log = fixture.tmp.path().join("stdout.log");
    let model = fixture.client.model.canonical();
    assert!(!crate::agent_backend::sdk_bridge_needs_restart(
        &fixture.client
    ));
    let started = fixture
        .client
        .ensure_coder_session(fixture.tmp.path())
        .await
        .expect("ensure again");
    assert!(
        !started.is_fresh(),
        "fresh bridge must report reuse, not a fresh agent context"
    );
    assert_eq!(
        fixture.started,
        bridge_started_at(&fixture.client),
        "fresh bridge must not restart"
    );
    assert_eq!(
        count_agent_start_log_lines(&stdout_log, &model),
        1,
        "reused open session must not emit another a| line"
    );
    end_fixture(fixture).await;
}

#[tokio::test]
async fn cursor_sdk_ensure_restarts_stale_bridge() {
    let _guard = crate::test_utils::test_env_lock();
    let mut fixture = open_ensure_fixture().await;
    let stdout_log = fixture.tmp.path().join("stdout.log");
    let model = fixture.client.model.canonical();
    backdate_bridge(&mut fixture.client, fixture.started);
    assert!(crate::agent_backend::sdk_bridge_needs_restart(
        &fixture.client
    ));
    let fresh_context = fixture
        .client
        .ensure_coder_session(fixture.tmp.path())
        .await
        .expect("ensure restart");
    assert!(
        !fresh_context.is_fresh(),
        "stale bridge resume continues the prior agent; do not send header.md again"
    );
    assert!(bridge_started_at(&fixture.client) > fixture.started);
    assert!(!crate::agent_backend::sdk_bridge_needs_restart(
        &fixture.client
    ));
    assert_eq!(
        count_agent_start_log_lines(&stdout_log, &model),
        1,
        "resume restart must not emit another a| line"
    );
    prompt_once(&mut fixture.client, &fixture.tmp.path().join("prompts.log")).await;
    end_fixture(fixture).await;
}
