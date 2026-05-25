use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use super::config::DeferredLogConfig;
use super::sink_build::build_tool_entry;
use super::sink::DeferredLogSink;
use super::types::{DeferredEntry, EnrichKey, TeeSinkMeta, ToolDrainMeta, ToolSummaryBuild};

pub type SharedDeferSink = Arc<std::sync::Mutex<DeferredLogSink>>;

pub fn zero_age_defer_shared(session: &str) -> SharedDeferSink {
    Arc::new(std::sync::Mutex::new(DeferredLogSink::new(
        session.to_string(),
        PathBuf::new(),
        DeferredLogConfig {
            max_age: Duration::from_millis(0),
            max_drain_per_log: 64,
            cursor_dir: PathBuf::new(),
        },
    )))
}

pub fn aged_defer_shared(session: &str) -> SharedDeferSink {
    Arc::new(std::sync::Mutex::new(DeferredLogSink::new(
        session.to_string(),
        PathBuf::new(),
        DeferredLogConfig {
            max_age: Duration::from_secs(3600),
            max_drain_per_log: 64,
            cursor_dir: PathBuf::new(),
        },
    )))
}

pub fn test_tool_entry(plain: &str) -> DeferredEntry {
    build_tool_entry(ToolSummaryBuild {
        tee: TeeSinkMeta {
            who: "w".to_string(),
            ts: "ts".to_string(),
            emit_stdout_markdown: false,
        },
        plain: plain.to_string(),
        display: plain.to_string(),
        enrich: None,
        meta: None,
    })
}

pub fn enrich_read_entry(tool_call_id: &str, fallback: &str) -> DeferredEntry {
    build_tool_entry(ToolSummaryBuild {
        tee: TeeSinkMeta {
            who: "w".to_string(),
            ts: "ts".to_string(),
            emit_stdout_markdown: false,
        },
        plain: String::new(),
        display: String::new(),
        enrich: Some(EnrichKey {
            tool_call_id: tool_call_id.to_string(),
            kind: "read".to_string(),
        }),
        meta: Some(ToolDrainMeta {
            tool_call_id: tool_call_id.to_string(),
            kind: "read".to_string(),
            elapsed: Duration::from_millis(1),
            raw_output: None,
            fallback_plain: fallback.to_string(),
        }),
    })
}

pub fn zero_age_config(cursor_dir: PathBuf, max_drain: usize) -> DeferredLogConfig {
    DeferredLogConfig {
        max_age: Duration::from_millis(0),
        max_drain_per_log: max_drain,
        cursor_dir,
    }
}

pub fn zero_age_sink(session: &str, work_dir: PathBuf, max_drain: usize) -> DeferredLogSink {
    DeferredLogSink::new(
        session.to_string(),
        work_dir.clone(),
        zero_age_config(work_dir, max_drain),
    )
}

pub fn capture_stdout_log(run: impl FnOnce()) -> String {
    let _guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let tmp = tempfile::tempdir().expect("tempdir");
    let log_path = tmp.path().join("stdout.log");
    crate::output::set_stdout_log_path(Some(log_path.clone()));
    run();
    crate::output::set_stdout_log_path(None);
    std::fs::read_to_string(log_path).unwrap_or_default()
}

pub(crate) fn capture_stdout_render_unlocked(run: impl FnOnce()) -> (String, String) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let log_path = tmp.path().join("stdout.log");
    crate::output::set_stdout_log_path(Some(log_path.clone()));
    crate::output::enable_stdout_capture();
    run();
    let terminal = crate::output::take_captured_stdout();
    crate::output::set_stdout_log_path(None);
    let log = std::fs::read_to_string(log_path).unwrap_or_default();
    (terminal, log)
}

pub fn capture_stdout_render(run: impl FnOnce()) -> (String, String) {
    let _guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    capture_stdout_render_unlocked(run)
}

pub struct DeferLogTestCtx {
    pub tmp: tempfile::TempDir,
    pub stdout_guard: std::sync::MutexGuard<'static, ()>,
    pub heartbeat_guard: std::sync::MutexGuard<'static, ()>,
    pub log_path: PathBuf,
    pub shared: SharedDeferSink,
}

pub fn defer_log_test_ctx(aged: bool) -> DeferLogTestCtx {
    let stdout_guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let heartbeat_guard = crate::output::HEARTBEAT_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    crate::output::reset_stdout_heartbeat_for_test();
    let tmp = tempfile::tempdir().expect("tempdir");
    let log_path = tmp.path().join("stdout.log");
    crate::output::set_stdout_log_path(Some(log_path.clone()));
    let shared = if aged {
        aged_defer_shared("sess")
    } else {
        zero_age_defer_shared("sess")
    };
    super::register_active_sink(Arc::clone(&shared));
    super::install_stdout_hooks();
    DeferLogTestCtx {
        tmp,
        stdout_guard,
        heartbeat_guard,
        log_path,
        shared,
    }
}
