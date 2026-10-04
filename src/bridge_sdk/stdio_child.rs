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

pub(crate) type SpawnedStdio = (Child, ChildStdin, ChildStdout, Option<u32>, HashSet<u32>);

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
        let (child, stdin, stdout, process_group_id, spawn_pid_baseline) = process;
        Self {
            child: AsyncMutex::new(Some(child)),
            stdin: Arc::new(AsyncMutex::new(stdin)),
            stdout: Arc::new(AsyncMutex::new(BufReader::new(stdout))),
            process_group_id,
            spawn_pid_baseline,
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
