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
        None,
    );
    assert!(session_active_for_test());
    clear_session();
    assert!(!session_active_for_test());
}
