use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use super::session::PiEmbeddedSession;

pub(crate) fn start_embedded_mem_watch(session: &PiEmbeddedSession) {
    #[cfg(unix)]
    {
        if crate::acp::test_no_real_agent_enabled() {
            return;
        }
        let reader_dead = Arc::clone(&session.reader_dead);
        let baseline = session.spawn_pid_baseline.clone();
        let work_dir = session.work_dir.clone();
        let run_dir = session.log.run_dir.clone();
        tokio::spawn(async move {
            watch_embedded_memory(reader_dead, baseline, work_dir, run_dir).await;
        });
    }
    #[cfg(not(unix))]
    {
        let _ = session;
    }
}

#[cfg(unix)]
async fn watch_embedded_memory(
    reader_dead: Arc<AtomicBool>,
    baseline: std::collections::HashSet<u32>,
    work_dir: std::path::PathBuf,
    run_dir: Option<std::path::PathBuf>,
) {
    let limit_bytes = crate::mem_limit_config::load_mem_limit_bytes(&work_dir);
    crate::acp::watch_process_group_memory(crate::acp::MemWatchHandles {
        reader_dead,
        pgid: None,
        limit_bytes,
        spawn_pid_baseline: baseline,
        run_dir,
    })
    .await;
}
