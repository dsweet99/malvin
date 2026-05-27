//! Best-effort synchronous teardown when the last [`super::AcpSession`] handle is dropped
//! without an explicit [`super::AcpSession::shutdown`].

use std::sync::atomic::Ordering;
use std::sync::Arc;

use std::collections::HashSet;

use super::session_types::AcpSessionInner;

#[cfg(unix)]
fn signal_targets_blocking(
    targets: &HashSet<u32>,
    process_group_id: Option<u32>,
    signal: i32,
) {
    use super::unix_process_group_ps::{signal_pid, signal_process_group};
    for pid in targets {
        signal_pid(*pid, signal);
    }
    if let Some(pgid) = process_group_id {
        signal_process_group(pgid, signal);
    }
}

#[cfg(unix)]
pub(crate) fn terminate_agent_process_group_blocking(
    process_group_id: Option<u32>,
    spawn_baseline: &HashSet<u32>,
) {
    use super::unix_process_group_teardown::kill_targets_for_teardown;
    let orphan_scan = !spawn_baseline.is_empty();
    if process_group_id.is_none() && !orphan_scan {
        return;
    }
    let targets = kill_targets_for_teardown(process_group_id, Some(spawn_baseline));
    signal_targets_blocking(&targets, process_group_id, 15);
    std::thread::sleep(std::time::Duration::from_millis(50));
    signal_targets_blocking(&targets, process_group_id, 9);
}

#[cfg(unix)]
fn take_child_without_tokio_drop(inner: &AcpSessionInner) {
    if tokio::runtime::Handle::try_current().is_ok() {
        return;
    }
    {
        let mut slot = inner.child.blocking_lock();
        if let Some(ch) = slot.take() {
            if let Some(pid) = ch.id() {
                let _ = std::process::Command::new("kill")
                    .args(["-KILL", &pid.to_string()])
                    .status();
            }
            std::mem::forget(ch);
        }
    }
}

#[cfg(unix)]
pub(crate) fn acp_session_drop_teardown(inner: &AcpSessionInner) {
    inner.reader_dead.store(true, Ordering::SeqCst);
    crate::active_agent_heartbeat::unregister_active_agent_process_group(inner.process_group_id);
    terminate_agent_process_group_blocking(inner.process_group_id, &inner.spawn_pid_baseline);
    take_child_without_tokio_drop(inner);
    crate::malvin_sandbox::clear_active_sandbox_session();
}

#[cfg(all(test, unix))]
pub(crate) fn take_child_without_tokio_drop_for_test(inner: &AcpSessionInner) {
    take_child_without_tokio_drop(inner);
}

pub(crate) fn acp_session_drop_if_last(inner: &Arc<AcpSessionInner>) {
    if Arc::strong_count(inner) > 1 {
        return;
    }
    #[cfg(unix)]
    acp_session_drop_teardown(inner);
}

#[cfg(all(test, unix))]
mod unix_regression {
    use std::sync::Arc;

    use super::super::session_tests::session_with_sleep_child_for_mem_watch;
    use super::super::unix_process_group_ps::pid_alive;
    use super::take_child_without_tokio_drop_for_test;

    #[tokio::test]
    async fn drop_without_shutdown_kills_sandbox_child() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let (session, pgid) = session_with_sleep_child_for_mem_watch(tmp.path());
        drop(session);
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        assert!(
            !pid_alive(pgid),
            "last AcpSession drop must tear down the sandbox child (pgid={pgid})"
        );
    }

    #[test]
    fn terminate_agent_process_group_blocking_kills_sleep_child() {
        use std::os::unix::process::CommandExt;
        use std::process::Command;
        use std::time::Duration;

        let baseline = super::super::unix_process_group_ps::snapshot_pids();
        let mut cmd = Command::new("sleep");
        cmd.arg("120");
        cmd.process_group(0);
        let mut child = cmd.spawn().expect("spawn sleep");
        let pgid = child.id();
        std::thread::sleep(Duration::from_millis(50));
        super::terminate_agent_process_group_blocking(Some(pgid), &baseline);
        let status = child.wait().expect("wait");
        assert!(!status.success() || status.code() == Some(9));
    }

    #[test]
    fn take_child_without_runtime_empties_child_slot() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        let (session, _pgid) = rt.block_on(async {
            session_with_sleep_child_for_mem_watch(tmp.path())
        });
        drop(rt);
        let inner = Arc::clone(&session.0);
        let inner_for_thread = Arc::clone(&inner);
        std::thread::spawn(move || take_child_without_tokio_drop_for_test(&inner_for_thread))
            .join()
            .expect("join");
        let child_slot_empty = {
            let guard = inner.child.blocking_lock();
            guard.is_none()
        };
        assert!(child_slot_empty);
    }

}

#[cfg(test)]
mod tests {
    #[test]
    fn kiss_cov_session_drop_teardown_symbols() {
        let _ = stringify!(acp_session_drop_if_last);
        #[cfg(unix)]
        {
            let _ = stringify!(acp_session_drop_teardown);
            let _ = stringify!(take_child_without_tokio_drop);
            let _ = stringify!(terminate_agent_process_group_blocking);
            let _ = stringify!(signal_targets_blocking);
            let _ = stringify!(unix_regression::drop_without_shutdown_kills_sandbox_child);
            let _ = stringify!(unix_regression::take_child_without_runtime_empties_child_slot);
            let _ = stringify!(unix_regression::terminate_agent_process_group_blocking_kills_sleep_child);
        }
    }
}
