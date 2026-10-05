use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use tokio::io::BufReader;
use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio::sync::Mutex as AsyncMutex;

use super::session_io::MemWatchArgs;
use super::stdio_teardown::StdioTeardown;
use super::stream_log::StreamLog;

pub(crate) struct SpawnedStdio {
    pub child: Child,
    pub stdin: ChildStdin,
    pub stdout: ChildStdout,
    pub pgid: Option<u32>,
    pub baseline_pids: HashSet<u32>,
}

impl SpawnedStdio {
    pub(crate) fn new(child: Child, stdin: ChildStdin, stdout: ChildStdout) -> Self {
        let pgid = child.id();
        Self {
            child,
            stdin,
            stdout,
            pgid,
            baseline_pids: HashSet::new(),
        }
    }
}

pub struct StdioChild {
    pub child: AsyncMutex<Option<Child>>,
    pub stdin: Arc<AsyncMutex<ChildStdin>>,
    pub stdout: Arc<AsyncMutex<BufReader<ChildStdout>>>,
    pub process_group_id: Option<u32>,
    pub spawn_pid_baseline: HashSet<u32>,
    pub reader_dead: Arc<AtomicBool>,
    pub work_dir: PathBuf,
    pub log: StreamLog,
}

impl StdioChild {
    #[must_use]
    pub(crate) fn new(process: SpawnedStdio, work_dir: PathBuf, log: StreamLog) -> Self {
        Self {
            child: AsyncMutex::new(Some(process.child)),
            stdin: Arc::new(AsyncMutex::new(process.stdin)),
            stdout: Arc::new(AsyncMutex::new(BufReader::new(process.stdout))),
            process_group_id: process.pgid,
            spawn_pid_baseline: process.baseline_pids,
            reader_dead: Arc::new(AtomicBool::new(false)),
            work_dir,
            log,
        }
    }

    #[must_use]
    pub(crate) fn mem_watch_args(&self) -> MemWatchArgs<'_> {
        MemWatchArgs {
            process_group_id: self.process_group_id,
            reader_dead: &self.reader_dead,
            work_dir: &self.work_dir,
            spawn_pid_baseline: &self.spawn_pid_baseline,
            run_dir: self.log.run_dir.as_deref(),
        }
    }

    pub(crate) async fn shutdown_kill_and_clear(&self) {
        self.teardown().shutdown_kill_and_clear().await;
    }

    fn teardown(&self) -> StdioTeardown<'_> {
        StdioTeardown::new(
            &self.child,
            self.process_group_id,
            &self.spawn_pid_baseline,
            &self.reader_dead,
        )
    }
}

impl Drop for StdioChild {
    fn drop(&mut self) {
        self.teardown().drop_teardown();
    }
}
