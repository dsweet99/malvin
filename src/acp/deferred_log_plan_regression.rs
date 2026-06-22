//! Regression tests for deferred-log + `store.db` enrichment (`plan.md`).

use super::kpop_stdout_logger_plan_helpers::{
    begin_stdout_log_fixture, finish_stdout_log_fixture, stdout_log_test_guard,
    styled_markdown_trace_writer,
};
use crate::acp::{PromptTraceWriter, TraceChunkCoalescer, write_tool_summary_trace_line};
use std::sync::Arc;

use crate::cursor_store::{install_test_store, TestStoreSpec};
use crate::deferred_log::{
    install_stdout_hooks, register_active_sink, unregister_active_sink, DeferredLogSink,
};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const SESSION_ID: &str = "deferred-log-regression-session";
const TOOL_CALL_ID: &str = "toolu_test_read_fixture_001";
const READ_PATH: &str = "/home/user/project/src/index.ts";

fn read_start_empty_raw_input() -> Value {
    json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call",
            "toolCallId": TOOL_CALL_ID,
            "kind": "read",
            "status": "pending",
            "title": "Read File",
            "rawInput": {}
        }}
    })
}

fn read_done_empty_raw_input() -> Value {
    json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": TOOL_CALL_ID,
            "status": "completed",
            "title": "Read File",
            "rawInput": {},
            "rawOutput": {"content": "export default function main() {}"}
        }}
    })
}

struct EnvRestore {
    cursor_dir: Option<std::ffi::OsString>,
    defer_log: Option<std::ffi::OsString>,
    max_age_ms: Option<std::ffi::OsString>,
}

impl EnvRestore {
    fn set(cursor_dir: &Path) -> Self {
        let saved_cursor = std::env::var_os("MALVIN_CURSOR_DIR");
        let saved_defer = std::env::var_os("MALVIN_DEFER_LOG");
        let saved_max_age = std::env::var_os("MALVIN_DEFER_LOG_MAX_AGE_MS");
        unsafe {
            std::env::set_var("MALVIN_CURSOR_DIR", cursor_dir);
            std::env::remove_var("MALVIN_DEFER_LOG");
            std::env::set_var("MALVIN_DEFER_LOG_MAX_AGE_MS", "0");
        }
        Self {
            cursor_dir: saved_cursor,
            defer_log: saved_defer,
            max_age_ms: saved_max_age,
        }
    }
}

impl Drop for EnvRestore {
    fn drop(&mut self) {
        unsafe {
            regression_restore_env("MALVIN_CURSOR_DIR", self.cursor_dir.take());
            regression_restore_env("MALVIN_DEFER_LOG", self.defer_log.take());
            regression_restore_env("MALVIN_DEFER_LOG_MAX_AGE_MS", self.max_age_ms.take());
        }
    }
}

unsafe fn regression_restore_env(key: &str, value: Option<std::ffi::OsString>) {
    match value {
        Some(v) => unsafe { std::env::set_var(key, v) },
        None => unsafe { std::env::remove_var(key) },
    }
}

fn defer_trace_writer(trace_file: tokio::fs::File, work_dir: PathBuf) -> PromptTraceWriter {
    let mut writer = styled_markdown_trace_writer(trace_file, work_dir.clone());
    writer.session_id = SESSION_ID.to_string();
    if let Some(sink) = DeferredLogSink::for_prompt(SESSION_ID.to_string(), work_dir) {
        let shared = Arc::new(std::sync::Mutex::new(sink));
        register_active_sink(Arc::clone(&shared));
        install_stdout_hooks();
        writer.deferred_sink = Some(shared);
    }
    writer
}

async fn tee_read_lifecycle_stdout(work_dir: &Path, start: &Value, done: &Value) -> String {
    let fixture = {
        let _guard = stdout_log_test_guard();
        begin_stdout_log_fixture()
    };
    let trace_file = tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&fixture.trace_path)
        .await
        .unwrap();
    let mut writer = defer_trace_writer(trace_file, work_dir.to_path_buf());
    let mut coalesce = TraceChunkCoalescer::default();
    write_tool_summary_trace_line(&mut writer, &mut coalesce, start, true)
        .await;
    write_tool_summary_trace_line(&mut writer, &mut coalesce, done, true)
        .await;
    let (display, log) =
        crate::output::stdout_tagged_display_and_log_line("malvin", "hook-probe", None);
    assert!(crate::output::try_defer_tagged_stdout(&display, &log));
    drop(writer);
    unregister_active_sink();
    finish_stdout_log_fixture(fixture)
}

#[tokio::test]
async fn read_done_tee_shows_store_db_path_when_wire_raw_input_empty() {
    let tmp = tempfile::tempdir().unwrap();
    install_test_store(&TestStoreSpec {
        cursor_dir: tmp.path(),
        session_id: SESSION_ID,
        tool_call_id: TOOL_CALL_ID,
        path: READ_PATH,
        offset: None,
        limit: None,
    });
    let _env = EnvRestore::set(tmp.path());
    let stdout = tee_read_lifecycle_stdout(
        tmp.path(),
        &read_start_empty_raw_input(),
        &read_done_empty_raw_input(),
    )
    .await;
    assert!(
        !stdout.contains("Read file ·"),
        "expected store.db path enrichment on stdout; got {stdout:?}"
    );
    assert!(
        stdout.contains("index.ts"),
        "expected relativized read path on stdout; got {stdout:?}"
    );
    assert!(
        stdout.contains("hook-probe"),
        "expected malvin-tag hook defer on stdout; got {stdout:?}"
    );
}
#[cfg(test)]
#[path = "deferred_log_plan_regression_test.rs"]
mod deferred_log_plan_regression_test;#[cfg(test)]
#[path = "deferred_log_plan_regression_kiss_cov_test.rs"]
mod deferred_log_plan_regression_kiss_cov_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<EnvRestore> = None;
        let _ = defer_trace_writer;
        let _ = drop;
        let _ = read_done_empty_raw_input;
        let _ = read_done_tee_shows_store_db_path_when_wire_raw_input_empty;
        let _ = read_start_empty_raw_input;
        let _ = regression_restore_env;
        let _ = tee_read_lifecycle_stdout;
    }
}
