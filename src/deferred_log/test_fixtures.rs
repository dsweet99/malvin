use std::path::PathBuf;
use std::time::Duration;

use super::config::DeferredLogConfig;
use super::sink::{build_tool_entry, DeferredLogSink};
use super::types::{DeferredEntry, EnrichKey, TeeSinkMeta, ToolDrainMeta, ToolSummaryBuild};

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
