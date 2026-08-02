//! Best-effort herdr session lifecycle (start / working / end).

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use super::env::HerdrEnv;
use super::request::{
    clear_agent_authority, clear_metadata_teardown, next_seq, report_agent, report_agent_session,
    report_metadata_sparse,
};
use super::send::{send_request, send_request_retry};

#[derive(Debug, Default)]
struct Session {
    active: bool,
    pane_id: String,
    socket_path: PathBuf,
    agent_session_id: Option<String>,
}

fn session_mutex() -> &'static Mutex<Session> {
    static SESSION: OnceLock<Mutex<Session>> = OnceLock::new();
    SESSION.get_or_init(|| Mutex::new(Session::default()))
}

/// Skip live I/O inside unit tests unless explicitly enabled (avoids herdr pane spam).
#[allow(clippy::missing_const_for_fn)] // non-test body is `true`; test body reads env.
fn live_io_allowed() -> bool {
    #[cfg(test)]
    {
        std::env::var_os("MALVIN_TEST_HERDR_IO").is_some()
    }
    #[cfg(not(test))]
    {
        true
    }
}

/// Bind pane to this malvin run and report `working`.
pub fn notify_run_start(run_dir: &Path) {
    let _ = std::panic::catch_unwind(|| notify_run_start_inner(run_dir));
}

/// Re-bind after an ACP/coder session starts (cursor hook may have stolen authority).
pub fn notify_reclaim() {
    let _ = std::panic::catch_unwind(notify_reclaim_inner);
}

fn notify_run_start_inner(run_dir: &Path) {
    if !live_io_allowed() {
        return;
    }
    let Some(env) = HerdrEnv::from_os_env() else {
        return;
    };
    let session_id = run_dir_session_id(run_dir);
    activate(&env, session_id.as_deref());
    emit_bind_reports(&env, session_id.as_deref());
}

fn notify_reclaim_inner() {
    if !live_io_allowed() {
        return;
    }
    let Some(snapshot) = active_snapshot() else {
        return;
    };
    let env = HerdrEnv {
        socket_path: snapshot.0,
        pane_id: snapshot.1,
    };
    emit_bind_reports(&env, snapshot.2.as_deref());
}

fn run_dir_session_id(run_dir: &Path) -> Option<String> {
    run_dir.file_name().and_then(|s| s.to_str()).map(str::to_string)
}

fn emit_bind_reports(env: &HerdrEnv, session_id: Option<&str>) {
    let pane = env.pane_id.as_str();
    let sock = env.socket_path.as_path();
    // Cursor ACP sessionStart hooks install full-lifecycle authority; clear first so malvin can bind.
    send_request(sock, &clear_agent_authority(pane, next_seq()));
    send_request(sock, &report_agent_session(pane, session_id, next_seq()));
    send_request(sock, &report_agent(pane, "working", session_id, next_seq()));
    send_request(sock, &report_metadata_sparse(pane, session_id, next_seq()));
}

fn activate(env: &HerdrEnv, agent_session_id: Option<&str>) {
    let mut guard = session_mutex()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    guard.active = true;
    guard.pane_id = env.pane_id.clone();
    guard.socket_path = env.socket_path.clone();
    guard.agent_session_id = agent_session_id.map(str::to_string);
}

/// Re-assert `working` (e.g. while `AgentPhase::Waiting` on shells/tools).
pub fn notify_working() {
    let _ = std::panic::catch_unwind(notify_working_inner);
}

fn notify_working_inner() {
    if !live_io_allowed() {
        return;
    }
    let Some(snapshot) = active_snapshot() else {
        return;
    };
    // Pulse only: start/reclaim own full bind; avoid re-clearing authority on every tool pulse.
    send_request(
        &snapshot.0,
        &report_agent(&snapshot.1, "working", snapshot.2.as_deref(), next_seq()),
    );
}

/// Report `idle` then clear display metadata. Idempotent; retries if a prior end failed to clear.
///
/// Deliberately does **not** call `pane.release_agent`: release leaves `agent_status=unknown`,
/// which keeps the activity presentation sticky. Staying bound as `idle` matches the required
/// post-run state; the next start still `clear_agent_authority` before rebinding.
pub fn notify_run_end() {
    let _ = std::panic::catch_unwind(notify_run_end_inner);
}

fn notify_run_end_inner() {
    if !live_io_allowed() {
        clear_session();
        return;
    }
    let Some(snapshot) = take_teardown_snapshot() else {
        return;
    };
    let idle = report_agent(&snapshot.1, "idle", snapshot.2.as_deref(), next_seq());
    let clear_meta = clear_metadata_teardown(&snapshot.1, next_seq());
    let idle_ok = send_request_retry(&snapshot.0, &idle);
    let clear_ok = send_request_retry(&snapshot.0, &clear_meta);
    if idle_ok && clear_ok {
        clear_session();
    }
    // If either send failed, keep pane credentials so a later `notify_run_end` can retry.
}

type Snapshot = (PathBuf, String, Option<String>);

fn active_snapshot() -> Option<Snapshot> {
    let guard = session_mutex()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if !guard.active {
        return None;
    }
    Some((
        guard.socket_path.clone(),
        guard.pane_id.clone(),
        guard.agent_session_id.clone(),
    ))
}

/// Snapshot for teardown: active bind, or retained credentials after a failed prior end.
fn take_teardown_snapshot() -> Option<Snapshot> {
    let mut guard = session_mutex()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if guard.pane_id.is_empty() || guard.socket_path.as_os_str().is_empty() {
        return None;
    }
    guard.active = false;
    Some((
        guard.socket_path.clone(),
        guard.pane_id.clone(),
        guard.agent_session_id.clone(),
    ))
}

fn clear_session() {
    let mut guard = session_mutex()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    *guard = Session::default();
}

#[cfg(test)]
pub(crate) fn reset_session_for_test() {
    clear_session();
}

#[cfg(test)]
pub(crate) fn session_active_for_test() -> bool {
    session_mutex()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .active
}

#[cfg(test)]
pub(crate) fn session_has_binding_for_test() -> bool {
    let guard = session_mutex()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    !guard.pane_id.is_empty() && !guard.socket_path.as_os_str().is_empty()
}

#[cfg(test)]
mod tests {
    use super::{
        activate, clear_session, live_io_allowed, notify_reclaim, notify_run_end, notify_run_start,
        notify_working, reset_session_for_test, session_active_for_test,
    };
    use crate::herdr::env::HerdrEnv;
    use std::path::PathBuf;

    #[test]
    fn notify_run_start_noops_without_env_triad() {
        reset_session_for_test();
        notify_run_start(std::path::Path::new("/tmp/fake-run-dir"));
        assert!(!session_active_for_test());
        let _ = live_io_allowed();
        let _ = notify_working;
        let _ = notify_reclaim;
        let _ = notify_run_end;
    }

    #[test]
    fn activate_and_clear_track_session_flag() {
        reset_session_for_test();
        activate(
            &HerdrEnv {
                socket_path: PathBuf::from("/tmp/x.sock"),
                pane_id: "pane".into(),
            },
            Some("run1"),
        );
        assert!(session_active_for_test());
        clear_session();
        assert!(!session_active_for_test());
    }
}
