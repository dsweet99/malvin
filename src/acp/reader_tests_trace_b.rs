use crate::acp::trace_line_write::TraceFileStdout;
use crate::acp::*;

async fn trace_file_write_line_prefixes_with_prompt_who() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("trace-prefix.log");
    let file = tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&path)
        .await
        .unwrap();
    let mut writer = PromptTraceWriter {
        file,
        who: "review_1".to_string(),
        plain_lines: false,
        stdout_replacement: None,
        placeholder_emitted: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: true,
        iterable_closed_warned: false,
    };
    crate::acp::trace_file_write_line(
        &mut writer,
        "hello",
        None,
        TraceFileStdout {
            tee_stdout: false,
            stream_iterable_closed: None,
            tee_line_override: None,
        },
    )
    .await;
    drop(writer);
    let s = tokio::fs::read_to_string(&path).await.unwrap();
    let inner = crate::output::format_log_tag_inner("review_1");
    assert!(
        s.contains(&format!(" [{inner}] hello\n")),
        "expected prompt-prefixed trace line, got {s:?}"
    );
}

#[tokio::test]
async fn raw_trace_file_write_line_records_thought_chunks_suppresses_thought_stdout_only() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("trace-raw-thought.log");
    let file = tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&path)
        .await
        .unwrap();
    let mut writer = PromptTraceWriter {
        file,
        who: "raw".to_string(),
        plain_lines: false,
        stdout_replacement: None,
        placeholder_emitted: false,
        raw_output: true,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        iterable_closed_warned: false,
    };
    crate::acp::trace_file_write_line(
        &mut writer,
        "internal reasoning",
        Some(SessionUpdateChunkKind::Thought),
        TraceFileStdout {
            tee_stdout: false,
            stream_iterable_closed: None,
            tee_line_override: None,
        },
    )
    .await;
    crate::acp::trace_file_write_line(
        &mut writer,
        "final answer",
        Some(SessionUpdateChunkKind::Message),
        TraceFileStdout {
            tee_stdout: false,
            stream_iterable_closed: None,
            tee_line_override: None,
        },
    )
    .await;
    drop(writer);
    let s = tokio::fs::read_to_string(&path).await.unwrap();
    assert!(
        s.contains("[internal reasoning]"),
        "trace file should record thought chunks when raw_output, got {s:?}"
    );
    assert!(
        s.contains("final answer"),
        "raw output should keep message chunks, got {s:?}"
    );
}

#[tokio::test]
async fn trace_file_write_line_plain_mode_omits_tag_prefix() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("trace-plain.log");
    let file = tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&path)
        .await
        .unwrap();
    let mut writer = PromptTraceWriter {
        file,
        who: "<do".to_string(),
        plain_lines: true,
        stdout_replacement: None,
        placeholder_emitted: false,
        raw_output: true,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        iterable_closed_warned: false,
    };
    crate::acp::trace_file_write_line(
        &mut writer,
        "assistant response",
        Some(SessionUpdateChunkKind::Message),
        TraceFileStdout {
            tee_stdout: false,
            stream_iterable_closed: None,
            tee_line_override: None,
        },
    )
    .await;
    drop(writer);
    let s = tokio::fs::read_to_string(&path).await.unwrap();
    assert_eq!(s, "assistant response\n");
}

#[tokio::test]
async fn trace_file_write_line_brackets_thought_chunks_in_trace_output() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("trace-thought.log");
    let file = tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&path)
        .await
        .unwrap();
    let mut writer = PromptTraceWriter {
        file,
        who: "review_1".to_string(),
        plain_lines: false,
        stdout_replacement: None,
        placeholder_emitted: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: true,
        iterable_closed_warned: false,
    };
    crate::acp::trace_file_write_line(
        &mut writer,
        "internal reasoning",
        Some(SessionUpdateChunkKind::Thought),
        TraceFileStdout {
            tee_stdout: false,
            stream_iterable_closed: None,
            tee_line_override: None,
        },
    )
    .await;
    drop(writer);
    let s = tokio::fs::read_to_string(&path).await.unwrap();
    assert!(
        s.contains("[internal reasoning]"),
        "thought chunks should be bracketed in traces, got {s:?}"
    );
}

#[tokio::test]
async fn trace_file_write_line_stdout_markdown_flag_tees_without_panic() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("trace-md-tee.log");
    let file = tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&path)
        .await
        .unwrap();
    let mut writer = PromptTraceWriter {
        file,
        who: "<kpop".to_string(),
        plain_lines: false,
        stdout_replacement: None,
        placeholder_emitted: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: true,
        iterable_closed_warned: false,
    };
    crate::acp::trace_file_write_line(
        &mut writer,
        "**x**",
        Some(SessionUpdateChunkKind::Message),
        TraceFileStdout {
            tee_stdout: true,
            stream_iterable_closed: None,
            tee_line_override: None,
        },
    )
    .await;
    drop(writer);
    let s = tokio::fs::read_to_string(&path).await.unwrap();
    assert!(
        s.contains("**x**"),
        "trace file keeps raw markdown regardless of stdout markdown flag: {s:?}"
    );
}
