//! `ensure_coder_session` age-refresh tests against `mock_bridge.js`.

use std::time::{Duration, Instant};

use super::client_mock_tests::{
    clear_mock_bridge_env, install_mock_bridge_env, mock_bridge_path, mock_client, prompt_once,
};
use super::session::SDK_BRIDGE_MAX_AGE;
use super::CursorSdkClient;

struct EnsureFixture {
    client: CursorSdkClient,
    tmp: tempfile::TempDir,
    started: Instant,
}

fn bridge_started_at(client: &CursorSdkClient) -> Instant {
    client.session.as_ref().expect("open").started_at
}

fn backdate_bridge(client: &mut CursorSdkClient, started: Instant) {
    client.session.as_mut().expect("open").started_at = started
        .checked_sub(SDK_BRIDGE_MAX_AGE + Duration::from_secs(1))
        .expect("backdate");
}

async fn open_ensure_fixture() -> EnsureFixture {
    install_mock_bridge_env(&mock_bridge_path());
    let tmp = tempfile::tempdir().expect("tmp");
    let mut client = mock_client(tmp.path());
    let _ = client.attach_run_timing_for_session();
    client
        .ensure_coder_session(tmp.path())
        .await
        .expect("ensure");
    let started = bridge_started_at(&client);
    EnsureFixture {
        client,
        tmp,
        started,
    }
}

async fn end_fixture(mut fixture: EnsureFixture) {
    fixture.client.end_coder_session().await.expect("end");
    clear_mock_bridge_env();
}

#[tokio::test]
async fn cursor_sdk_ensure_reuses_fresh_bridge() {
    let _guard = crate::test_utils::test_env_lock();
    let mut fixture = open_ensure_fixture().await;
    assert!(!fixture.client.sdk_bridge_needs_restart());
    fixture
        .client
        .ensure_coder_session(fixture.tmp.path())
        .await
        .expect("ensure again");
    assert_eq!(
        fixture.started,
        bridge_started_at(&fixture.client),
        "fresh bridge must not restart"
    );
    end_fixture(fixture).await;
}

#[tokio::test]
async fn cursor_sdk_ensure_restarts_stale_bridge() {
    let _guard = crate::test_utils::test_env_lock();
    let mut fixture = open_ensure_fixture().await;
    backdate_bridge(&mut fixture.client, fixture.started);
    assert!(fixture.client.sdk_bridge_needs_restart());
    fixture
        .client
        .ensure_coder_session(fixture.tmp.path())
        .await
        .expect("ensure restart");
    assert!(bridge_started_at(&fixture.client) > fixture.started);
    assert!(!fixture.client.sdk_bridge_needs_restart());
    prompt_once(
        &mut fixture.client,
        &fixture.tmp.path().join("prompts.log"),
    )
    .await;
    end_fixture(fixture).await;
}
