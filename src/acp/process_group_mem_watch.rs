use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use tracing::warn;

use super::session_types::AcpSession;

const POLL_INTERVAL: Duration = Duration::from_millis(500);

pub struct MemWatchHandles {
    pub reader_dead: Arc<std::sync::atomic::AtomicBool>,
    pub pgid: u32,
    pub limit_bytes: u64,
    pub spawn_pid_baseline: HashSet<u32>,
}

pub(crate) fn spawn_process_group_memory_watcher(session: &AcpSession, work_dir: &Path) {
    #[cfg(unix)]
    {
        let limit_bytes = crate::mem_limit_config::load_mem_limit_bytes(work_dir);
        let Some(pgid) = session.0.process_group_id else {
            return;
        };
        let handles = MemWatchHandles {
            reader_dead: Arc::clone(&session.0.reader_dead),
            pgid,
            limit_bytes,
            spawn_pid_baseline: session.0.spawn_pid_baseline.clone(),
        };
        tokio::spawn(async move {
            watch_process_group_memory(handles).await;
        });
    }
    #[cfg(not(unix))]
    {
        let _ = (session, work_dir);
    }
}

#[cfg(unix)]
pub async fn watch_process_group_memory(handles: MemWatchHandles) {
    let MemWatchHandles {
        pgid,
        limit_bytes,
        spawn_pid_baseline,
        ..
    } = handles;
    loop {
        if !crate::malvin_sandbox::sandbox_still_alive(Some(pgid), &spawn_pid_baseline) {
            return;
        }
        if let Some(rss) =
            crate::malvin_sandbox::malvin_session_rss_bytes(Some(pgid), &spawn_pid_baseline)
        {
            if rss > limit_bytes {
                warn!(
                    rss_bytes = rss,
                    limit_bytes,
                    pgid,
                    "malvin sandbox exceeded memory limit; terminating"
                );
                crate::acp::unix_process_group_teardown::terminate_agent_process_group(
                    Some(pgid),
                    &spawn_pid_baseline,
                )
                .await;
                return;
            }
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}
