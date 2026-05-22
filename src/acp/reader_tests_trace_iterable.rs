use crate::acp::trace_line_write::TraceFileStdout;
use crate::acp::*;

fn kpop_trace_writer(file: tokio::fs::File) -> PromptTraceWriter {
    PromptTraceWriter {
        file,
        who: "kpop".to_string(),
        plain_lines: false,
        stdout_replacement: None,
        placeholder_emitted: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        iterable_closed_warned: false,
    }
}

fn assert_iterable_closed_operational_stderr(stderr: &str, trace: &str) {
    assert!(
        trace.contains("WritableIterable is closed"),
        "trace file should still record agent text: {trace:?}"
    );
    assert!(
        stderr.contains(crate::output::WARNING_WHO)
            && stderr.contains("acp: WritableIterable is closed"),
        "operational warning must use warning who, got: {stderr:?}"
    );
    assert!(
        !stderr.contains("[<kpop"),
        "iterable-closed must not be tee'd with session who: {stderr:?}"
    );
}

fn session_update_message_chunk_json(text: &str) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "method": "session/update",
        "params": {
            "sessionId": "x",
            "update": {
                "sessionUpdate": "agent_message_chunk",
                "content": { "type": "text", "text": text }
            }
        }
    })
}

async fn open_kpop_trace_writer(path: &std::path::Path) -> PromptTraceWriter {
    let file = tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)
        .await
        .unwrap();
    kpop_trace_writer(file)
}

async fn flush_coalesce_lines(
    writer: &mut PromptTraceWriter,
    coalesce: &mut TraceChunkCoalescer,
    tee_stdout: bool,
) {
    for (kind, tl, stream) in coalesce.flush_all() {
        crate::acp::trace_file_write_line(
            writer,
            &tl,
            Some(kind),
            TraceFileStdout {
                tee_stdout,
                stream_iterable_closed: stream,
            },
        )
        .await;
    }
}

async fn deliver_coalesced_message_chunk(
    writer: &mut PromptTraceWriter,
    text: &str,
    tee_stdout: bool,
) -> TraceChunkCoalescer {
    let parsed = session_update_message_chunk_json(text);
    let raw = serde_json::to_string(&parsed).unwrap();
    let mut coalesce = TraceChunkCoalescer::default();
    crate::acp::trace_line_write::write_trace_line_coalesced(
        writer,
        &mut coalesce,
        crate::acp::trace_line_write::WriteTraceLineCoalescedOpts {
            parsed: Some(&parsed),
            raw_line: &raw,
            tee_stdout,
        },
    )
    .await;
    coalesce
}

fn assert_split_iterable_closed_operational(stderr: &str, stdout_log: &str) {
    let kpop_tag = format!("[{}]", crate::output::format_log_tag_inner("kpop"));
    assert!(
        stderr.contains(crate::output::WARNING_WHO)
            && stderr.contains("acp: WritableIterable is closed"),
        "split iterable-closed delivery must emit one operational warning, got: {stderr:?}"
    );
    assert!(
        !stdout_log.contains(&kpop_tag),
        "no coalesced fragment may tee with session who when stream contains iterable-closed, log: {stdout_log:?}"
    );
}

async fn run_split_iterable_closed_fixture() -> (String, String) {
    let text = format!("{}\n\nError: T: WritableIterable is closed", "p".repeat(95));
    let dir = tempfile::tempdir().unwrap();
    let stdout_path = dir.path().join("stdout-split.log");
    crate::output::set_stdout_log_path(Some(stdout_path.clone()));
    let mut writer = open_kpop_trace_writer(&dir.path().join("trace-iterable-split.log")).await;
    crate::output::clear_captured_stderr_lines();
    let mut coalesce = deliver_coalesced_message_chunk(&mut writer, &text, true).await;
    flush_coalesce_lines(&mut writer, &mut coalesce, true).await;
    drop(writer);
    crate::output::set_stdout_log_path(None);
    (
        crate::output::take_captured_stderr_lines().join(""),
        std::fs::read_to_string(&stdout_path).unwrap(),
    )
}

#[tokio::test]
async fn trace_file_write_line_iterable_closed_warns_without_kpop_tee() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("trace-iterable-closed.log");
    let file = tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&path)
        .await
        .unwrap();
    let mut writer = kpop_trace_writer(file);
    crate::output::clear_captured_stderr_lines();
    crate::acp::trace_file_write_line(
        &mut writer,
        "\n\nError: T: WritableIterable is closed",
        Some(SessionUpdateChunkKind::Message),
        TraceFileStdout {
            tee_stdout: true,
            stream_iterable_closed: None,
        },
    )
    .await;
    drop(writer);
    assert_iterable_closed_operational_stderr(
        &crate::output::take_captured_stderr_lines().join(""),
        &tokio::fs::read_to_string(&path).await.unwrap(),
    );
}

#[tokio::test]
async fn readable_iterable_closed_split_coalesce_emits_readable_operational_warning() {
    let _guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let text = format!("{}\n\nError: T: ReadableIterable is closed", "p".repeat(95));
    let dir = tempfile::tempdir().unwrap();
    let stdout_path = dir.path().join("stdout-readable-split.log");
    crate::output::set_stdout_log_path(Some(stdout_path.clone()));
    let mut writer = open_kpop_trace_writer(&dir.path().join("trace-readable-split.log")).await;
    crate::output::clear_captured_stderr_lines();
    let mut coalesce = deliver_coalesced_message_chunk(&mut writer, &text, true).await;
    flush_coalesce_lines(&mut writer, &mut coalesce, true).await;
    drop(writer);
    crate::output::set_stdout_log_path(None);
    let stderr = crate::output::take_captured_stderr_lines().join("");
    assert!(
        stderr.contains(crate::output::WARNING_WHO)
            && stderr.contains("acp: ReadableIterable is closed"),
        "readable iterable-closed coalesce stream must emit readable operational warning, got: {stderr:?}"
    );
    assert!(
        !stderr.contains("acp: WritableIterable is closed"),
        "readable stream-flag path must not mislabel as writable, got: {stderr:?}"
    );
}

#[tokio::test]
async fn iterable_closed_split_across_coalesce_emissions_suppresses_kpop_tee() {
    let _guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let (stderr, stdout_log) = run_split_iterable_closed_fixture().await;
    assert_split_iterable_closed_operational(&stderr, &stdout_log);
}
