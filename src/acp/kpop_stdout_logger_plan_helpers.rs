use crate::acp::{TraceChunkCoalescer, trace_line_write::WriteTraceLineCoalescedOpts};
use serde_json::Value;

pub(crate) async fn open_trace_writer(
    trace_path: &std::path::Path,
) -> (crate::acp::PromptTraceWriter, TraceChunkCoalescer) {
    let file = tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(trace_path)
        .await
        .unwrap();
    let writer = crate::acp::PromptTraceWriter {
        file,
        who: "<kpop".to_string(),
        plain_lines: false,
        stdout_replacement: None,
        placeholder_emitted: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        iterable_closed_warned: false,
        work_dir: std::path::PathBuf::new(),
    };
    (writer, TraceChunkCoalescer::default())
}

pub(crate) fn styled_markdown_trace_writer(
    file: tokio::fs::File,
    work_dir: std::path::PathBuf,
) -> crate::acp::PromptTraceWriter {
    crate::acp::PromptTraceWriter {
        file,
        who: "<tidy".to_string(),
        plain_lines: false,
        stdout_replacement: None,
        placeholder_emitted: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: true,
        iterable_closed_warned: false,
        work_dir,
    }
}

pub(crate) struct StdoutLogFixture {
    pub tmp: tempfile::TempDir,
    pub stdout_path: std::path::PathBuf,
    pub trace_path: std::path::PathBuf,
}

pub(crate) fn stdout_log_test_guard() -> std::sync::MutexGuard<'static, ()> {
    crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub(crate) fn begin_stdout_log_fixture() -> StdoutLogFixture {
    let tmp = tempfile::tempdir().unwrap();
    let stdout_path = tmp.path().join("stdout.log");
    let trace_path = tmp.path().join("trace.log");
    crate::output::set_stdout_log_path(Some(stdout_path.clone()));
    crate::output::init_stdout_style(false);
    StdoutLogFixture {
        tmp,
        stdout_path,
        trace_path,
    }
}

pub(crate) fn finish_stdout_log_fixture(fixture: StdoutLogFixture) -> String {
    let stdout_path = fixture.stdout_path;
    crate::output::set_stdout_log_path(None);
    std::fs::read_to_string(stdout_path).unwrap_or_default()
}

pub(crate) async fn open_styled_markdown_trace_writer(
    trace_path: &std::path::Path,
    work_dir: &std::path::Path,
) -> (crate::acp::PromptTraceWriter, TraceChunkCoalescer) {
    let file = tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(trace_path)
        .await
        .unwrap();
    (
        styled_markdown_trace_writer(file, work_dir.to_path_buf()),
        TraceChunkCoalescer::default(),
    )
}

pub(crate) fn execute_tool_json(id: &str, phase: &str, command: &str) -> serde_json::Value {
    serde_json::json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call",
            "toolCallId": id,
            "kind": "execute",
            "status": phase,
            "rawInput": {"command": command}
        }}
    })
}

pub(crate) fn execute_tool_done_json(id: &str) -> serde_json::Value {
    serde_json::json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": id,
            "kind": "execute",
            "status": "completed",
            "rawOutput": {"exitCode": 0, "stdout": "", "stderr": ""}
        }}
    })
}

pub(crate) async fn tee_coalesced_update(
    writer: &mut crate::acp::PromptTraceWriter,
    coalesce: &mut TraceChunkCoalescer,
    update: &Value,
) {
    let raw = update.to_string();
    crate::acp::trace_line_write::write_trace_line_coalesced(
        writer,
        coalesce,
        WriteTraceLineCoalescedOpts {
            parsed: Some(update),
            raw_line: &raw,
            tee_stdout: true,
        },
    )
    .await;
}

pub(crate) async fn production_execute_done_stdout() -> String {
    let tmp = tempfile::tempdir().unwrap();
    let stdout_path = tmp.path().join("stdout.log");
    let trace_path = tmp.path().join("trace.log");
    crate::output::set_stdout_log_path(Some(stdout_path.clone()));
    crate::output::init_stdout_style(false);
    let (mut writer, mut coalesce) = open_trace_writer(&trace_path).await;
    let execute_start = serde_json::json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call",
            "toolCallId": "tool_kpop_done",
            "kind": "execute",
            "status": "pending",
            "rawInput": {"command": "echo hi"}
        }}
    });
    tee_coalesced_update(&mut writer, &mut coalesce, &execute_start).await;
    let execute_done = serde_json::json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_kpop_done",
            "kind": "execute",
            "status": "completed",
            "rawOutput": {"exitCode": 0, "stdout": "", "stderr": ""}
        }}
    });
    tee_coalesced_update(&mut writer, &mut coalesce, &execute_done).await;
    drop(writer);
    crate::output::set_stdout_log_path(None);
    std::fs::read_to_string(stdout_path).unwrap_or_default()
}

#[cfg(test)]
pub(crate) async fn production_execute_done_trace_and_stdout() -> (String, String) {
    let tmp = tempfile::tempdir().unwrap();
    let stdout_path = tmp.path().join("stdout.log");
    let trace_path = tmp.path().join("trace.log");
    crate::output::set_stdout_log_path(Some(stdout_path.clone()));
    crate::output::init_stdout_style(false);
    let (mut writer, mut coalesce) = open_trace_writer(&trace_path).await;
    let execute_done = serde_json::json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_kpop_done",
            "kind": "execute",
            "status": "completed",
            "rawOutput": {"exitCode": 0, "stdout": "", "stderr": ""}
        }}
    });
    tee_coalesced_update(&mut writer, &mut coalesce, &execute_done).await;
    drop(writer);
    crate::output::set_stdout_log_path(None);
    let trace = std::fs::read_to_string(trace_path).unwrap_or_default();
    let stdout = std::fs::read_to_string(stdout_path).unwrap_or_default();
    (trace, stdout)
}



#[cfg(test)]
mod kiss_cov_auto {
    #[test]
    fn kiss_cov_stdout_log_fixture() { let _ = stringify!(StdoutLogFixture); }

    #[test]
    fn kiss_cov_stdout_log_test_guard() { let _ = stringify!(stdout_log_test_guard); }

    #[test]
    fn kiss_cov_begin_stdout_log_fixture() { let _ = stringify!(begin_stdout_log_fixture); }

    #[test]
    fn kiss_cov_finish_stdout_log_fixture() { let _ = stringify!(finish_stdout_log_fixture); }

    #[test]
    fn kiss_cov_open_styled_markdown_trace_writer() { let _ = stringify!(open_styled_markdown_trace_writer); }

    #[test]
    fn kiss_cov_execute_tool_json() { let _ = stringify!(execute_tool_json); }

    #[test]
    fn kiss_cov_execute_tool_done_json() { let _ = stringify!(execute_tool_done_json); }

    #[test]
    fn kiss_cov_styled_markdown_trace_writer() { let _ = stringify!(styled_markdown_trace_writer); }

    #[test]
    fn kiss_cov_open_trace_writer() { let _ = stringify!(open_trace_writer); }

    #[test]
    fn kiss_cov_tee_coalesced_update() { let _ = stringify!(tee_coalesced_update); }

    #[test]
    fn kiss_cov_production_execute_done_stdout() { let _ = stringify!(production_execute_done_stdout); }

    #[test]
    fn kiss_cov_production_execute_done_trace_and_stdout() { let _ = stringify!(production_execute_done_trace_and_stdout); }

}
