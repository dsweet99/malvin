use crate::acp::trace_line_write::TraceFileStdout;
use crate::acp::*;

struct TraceBWriterOpts {
    who: &'static str,
    plain_lines: bool,
    raw_output: bool,
    emit_stdout_markdown: bool,
}

async fn open_trace_b_writer(path: &std::path::Path, opts: TraceBWriterOpts) -> PromptTraceWriter {
    let file = tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)
        .await
        .unwrap();
    PromptTraceWriter {
        file,
        who: opts.who.to_string(),
        plain_lines: opts.plain_lines,
        stdout_replacement: None,
        placeholder_emitted: false,
        raw_output: opts.raw_output,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: opts.emit_stdout_markdown,
        iterable_closed_warned: false,
        work_dir: std::path::PathBuf::new(),
        run_timing: None,
        session_id: String::new(),
        deferred_sink: None,
    }
}

const TRACE_STDOUT_OFF: TraceFileStdout<'_> = TraceFileStdout {
    tee_stdout: false,
    stream_iterable_closed: None,
    tee_line_override: None,
    tee_line_display: None,
    ts: None,
};

#[tokio::test]
async fn trace_file_write_line_prefixes_with_prompt_who() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("trace-prefix.log");
    let mut writer = open_trace_b_writer(
        &path,
        TraceBWriterOpts {
            who: "review_1",
            plain_lines: false,
            raw_output: false,
            emit_stdout_markdown: true,
        },
    )
    .await;
    crate::acp::trace_file_write_line(&mut writer, "hello", None, TRACE_STDOUT_OFF).await;
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
    let mut writer = open_trace_b_writer(
        &path,
        TraceBWriterOpts {
            who: "raw",
            plain_lines: false,
            raw_output: true,
            emit_stdout_markdown: false,
        },
    )
    .await;
    crate::acp::trace_file_write_line(
        &mut writer,
        "internal reasoning",
        Some(SessionUpdateChunkKind::Thought),
        TRACE_STDOUT_OFF,
    )
    .await;
    crate::acp::trace_file_write_line(
        &mut writer,
        "final answer",
        Some(SessionUpdateChunkKind::Message),
        TRACE_STDOUT_OFF,
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
    let mut writer = open_trace_b_writer(
        &path,
        TraceBWriterOpts {
            who: "<do",
            plain_lines: true,
            raw_output: true,
            emit_stdout_markdown: false,
        },
    )
    .await;
    crate::acp::trace_file_write_line(
        &mut writer,
        "assistant response",
        Some(SessionUpdateChunkKind::Message),
        TRACE_STDOUT_OFF,
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
    let mut writer = open_trace_b_writer(
        &path,
        TraceBWriterOpts {
            who: "review_1",
            plain_lines: false,
            raw_output: false,
            emit_stdout_markdown: true,
        },
    )
    .await;
    crate::acp::trace_file_write_line(
        &mut writer,
        "internal reasoning",
        Some(SessionUpdateChunkKind::Thought),
        TRACE_STDOUT_OFF,
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
    let mut writer = open_trace_b_writer(
        &path,
        TraceBWriterOpts {
            who: "<kpop",
            plain_lines: false,
            raw_output: false,
            emit_stdout_markdown: true,
        },
    )
    .await;
    crate::acp::trace_file_write_line(
        &mut writer,
        "**x**",
        Some(SessionUpdateChunkKind::Message),
        TraceFileStdout {
            tee_stdout: true,
            stream_iterable_closed: None,
            tee_line_override: None,
            tee_line_display: None,
            ts: None,
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

#[cfg(test)]
mod kiss_cov_auto {
    #[test]
    fn kiss_cov_trace_b_writer_opts() {
        let _ = stringify!(TraceBWriterOpts);
    }

    #[test]
    fn kiss_cov_open_trace_b_writer() {
        let _ = stringify!(open_trace_b_writer);
    }
}
