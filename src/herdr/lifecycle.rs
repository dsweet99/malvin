//! Best-effort herdr session lifecycle (start / working / end).
//!
//! Coexistence: ACP children are stripped of `HERDR_*` (`strip_herdr_env_from_child`)
//! so cursor hooks do not race malvin's parent reporter; `notify_reclaim` is backup.
//! Product path: reporter-only (not a Herdr kind / integration).

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use super::bind::emit_bind_reports;
use super::env::HerdrEnv;
use super::request::{clear_metadata_teardown, next_seq, report_agent};
use super::send::{send_request, send_request_checked};
use super::trace::log_herdr_failure;
use serde_json::Value;

#[derive(Debug, Default)]
struct Session {
    active: bool,
    pane_id: String,
    socket_path: PathBuf,
    agent_session_id: Option<String>,
    run_dir: Option<PathBuf>,
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
    activate(&env, session_id.as_deref(), Some(run_dir));
    emit_bind_reports(&env, session_id.as_deref(), Some(run_dir));
}

fn notify_reclaim_inner() {
    if !live_io_allowed() {
        return;
    }
    let Some(snap) = active_snapshot() else {
        return;
    };
    let env = HerdrEnv {
        socket_path: snap.socket_path,
        pane_id: snap.pane_id,
    };
    emit_bind_reports(&env, snap.agent_session_id.as_deref(), snap.run_dir.as_deref());
}

fn run_dir_session_id(run_dir: &Path) -> Option<String> {
    run_dir.file_name().and_then(|s| s.to_str()).map(str::to_string)
}

fn activate(env: &HerdrEnv, agent_session_id: Option<&str>, run_dir: Option<&Path>) {
    let mut guard = session_mutex()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    guard.active = true;
    guard.pane_id = env.pane_id.clone();
    guard.socket_path = env.socket_path.clone();
    guard.agent_session_id = agent_session_id.map(str::to_string);
    guard.run_dir = run_dir.map(Path::to_path_buf);
}

/// Re-assert `working` (e.g. while `AgentPhase::Waiting` on shells/tools).
pub fn notify_working() {
    let _ = std::panic::catch_unwind(notify_working_inner);
}

fn notify_working_inner() {
    if !live_io_allowed() {
        return;
    }
    let Some(snap) = active_snapshot() else {
        return;
    };
    // Pulse only: start/reclaim own full bind; avoid re-clearing authority on every tool pulse.
    send_request(
        &snap.socket_path,
        &report_agent(
            &snap.pane_id,
            "working",
            snap.agent_session_id.as_deref(),
            next_seq(),
        ),
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
    let Some(snap) = take_teardown_snapshot() else {
        return;
    };
    let idle = report_agent(
        &snap.pane_id,
        "idle",
        snap.agent_session_id.as_deref(),
        next_seq(),
    );
    let clear_meta = clear_metadata_teardown(&snap.pane_id, next_seq());
    let idle_ok = send_end_retry(&snap.socket_path, snap.run_dir.as_deref(), "end-idle", &idle);
    let clear_ok =
        send_end_retry(&snap.socket_path, snap.run_dir.as_deref(), "end-clear", &clear_meta);
    if idle_ok && clear_ok {
        clear_session();
    }
}

/// Best-effort teardown send with one retry; log the last error into the run dir.
fn send_end_retry(sock: &Path, run_dir: Option<&Path>, phase: &str, request: &Value) -> bool {
    if send_request_checked(sock, request).is_ok() {
        return true;
    }
    match send_request_checked(sock, request) {
        Ok(()) => true,
        Err(detail) => {
            log_herdr_failure(run_dir, phase, &detail);
            false
        }
    }
}

struct Snapshot {
    socket_path: PathBuf,
    pane_id: String,
    agent_session_id: Option<String>,
    run_dir: Option<PathBuf>,
}

fn active_snapshot() -> Option<Snapshot> {
    let guard = session_mutex()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if !guard.active {
        return None;
    }
    Some(Snapshot {
        socket_path: guard.socket_path.clone(),
        pane_id: guard.pane_id.clone(),
        agent_session_id: guard.agent_session_id.clone(),
        run_dir: guard.run_dir.clone(),
    })
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
    Some(Snapshot {
        socket_path: guard.socket_path.clone(),
        pane_id: guard.pane_id.clone(),
        agent_session_id: guard.agent_session_id.clone(),
        run_dir: guard.run_dir.clone(),
    })
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
#[path = "lifecycle_unit_tests.rs"]
mod lifecycle_unit_tests;
