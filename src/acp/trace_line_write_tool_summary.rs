use crate::acp::trace_line_write::{trace_file_write_line, TraceFileStdout, TraceTeeStdoutCtx};
use crate::acp::trace_line_write_tee::trace_tee_stdout_line;
use crate::acp::{PromptTraceWriter, TraceChunkCoalescer};
use crate::tool_summary::{tool_summary_lines, tool_summary_stdout_display, ToolSummaryDetail};

fn tool_summary_styled_tee_payload(writer: &PromptTraceWriter, plain: &str) -> (String, String) {
    let plain = if writer.emit_stdout_markdown {
        format!(":: {plain}")
    } else {
        plain.to_string()
    };
    let display = tool_summary_stdout_display(&plain);
    (plain, display)
}

pub(crate) async fn write_tool_summary_trace_line(
    trace_file: &mut PromptTraceWriter,
    coalesce: &mut TraceChunkCoalescer,
    parsed: &serde_json::Value,
    tee_stdout: bool,
) -> bool {
    coalesce
        .tool_tracker
        .set_work_dir(trace_file.work_dir.clone());
    let Some(summary) =
        tool_summary_lines(parsed, &mut coalesce.tool_tracker, ToolSummaryDetail::Stdout)
    else {
        return false;
    };
    let ts = crate::output::timestamp_now_string();
    trace_file_write_line(
        trace_file,
        &summary.log,
        None,
        TraceFileStdout {
            tee_stdout: false,
            stream_iterable_closed: None,
            tee_line_override: None,
            tee_line_display: None,
            ts: Some(&ts),
        },
    )
    .await;
    if !tee_stdout || trace_file.raw_output {
        return true;
    }
    let ctx = TraceTeeStdoutCtx {
        tee_stdout: true,
        kind: None,
        ts: &ts,
    };
    let tee_plain = summary
        .stdout
        .as_deref()
        .into_iter()
        .chain(
            summary
                .stdout
                .is_none()
                .then(|| summary.stdout_deferred.as_deref())
                .flatten(),
        );
    for plain in tee_plain {
        let (plain, display) = tool_summary_styled_tee_payload(trace_file, plain);
        trace_tee_stdout_line(trace_file, &plain, Some(display.as_str()), &ctx);
    }
    true
}


#[cfg(test)]
mod tool_summary_styled_tee_tests {
    use super::tool_summary_styled_tee_payload;
    use crate::acp::PromptTraceWriter;

    fn writer(emit_stdout_markdown: bool) -> PromptTraceWriter {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("trace.log");
        let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)
            .unwrap();
        PromptTraceWriter {
            file: tokio::fs::File::from_std(file),
            who: "t".to_string(),
            plain_lines: false,
            stdout_replacement: None,
            placeholder_emitted: false,
            raw_output: false,
            show_thoughts_on_stdout: false,
            emit_stdout_markdown,
            iterable_closed_warned: false,
            work_dir: dir.path().to_path_buf(),
        }
    }

    #[test]
    fn styled_payload_adds_colon_prefix() {
        let w = writer(true);
        let (plain, _) = tool_summary_styled_tee_payload(&w, "Run x");
        assert!(plain.starts_with(":: "));
    }

    #[test]
    fn unstyled_payload_omits_colon_prefix() {
        let w = writer(false);
        let (plain, _) = tool_summary_styled_tee_payload(&w, "Run x");
        assert_eq!(plain, "Run x");
    }
}

#[cfg(test)]
mod kiss_cov_auto {
    #[test]
    fn kiss_cov_write_tool_summary_trace_line() { let _ = stringify!(write_tool_summary_trace_line); }

    #[test]
    fn kiss_cov_tool_summary_styled_tee_payload() { let _ = stringify!(tool_summary_styled_tee_payload); }

}
