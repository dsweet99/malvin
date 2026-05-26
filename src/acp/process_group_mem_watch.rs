use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use tracing::warn;

use super::session_types::AcpSession;
use super::unix_process_group;

const POLL_INTERVAL: Duration = Duration::from_secs(3);

pub(super) struct MemWatchHandles {
    pub reader_dead: Arc<std::sync::atomic::AtomicBool>,
    pub pgid: u32,
    pub limit_bytes: u64,
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
fn process_group_still_alive(pgid: u32) -> bool {
    crate::process_group_rss::process_group_rss_bytes(pgid).is_some()
}

#[cfg(unix)]
pub(super) async fn watch_process_group_memory(handles: MemWatchHandles) {
    let MemWatchHandles {
        reader_dead,
        pgid,
        limit_bytes,
    } = handles;
    loop {
        tokio::time::sleep(POLL_INTERVAL).await;
        if reader_dead.load(Ordering::SeqCst) {
            return;
        }
        if !process_group_still_alive(pgid) {
            return;
        }
        let Some(rss) = crate::process_group_rss::process_group_rss_bytes(pgid) else {
            continue;
        };
        if rss <= limit_bytes {
            continue;
        }
        warn!(
            rss_bytes = rss,
            limit_bytes,
            pgid,
            "agent process group exceeded memory limit; terminating"
        );
        unix_process_group::terminate_process_group(Some(pgid)).await;
        return;
    }
}
