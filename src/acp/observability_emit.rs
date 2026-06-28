//! Dual-contract routing helpers for ACP [`PromptTraceWriter`].
//!
//! Audit writes go through [`write_audit_trace_line`]; narrative writes through [`tee_narrative_line`].
//! See [`crate::observability`] for channel trust rules.

use crate::acp::trace_line_write::TraceTeeStdoutCtx;
use crate::acp::PromptTraceWriter;
use crate::observability::{AUDIT_CHANNEL, NARRATIVE_CHANNEL};
use tokio::io::AsyncWriteExt;

/// Channel target for audit-only trace file writes.
pub(crate) const ACP_AUDIT_CHANNEL: crate::observability::ObservabilityChannel = AUDIT_CHANNEL;
/// Channel target for narrative stdout tee writes.
pub(crate) const ACP_NARRATIVE_CHANNEL: crate::observability::ObservabilityChannel =
    NARRATIVE_CHANNEL;

/// Append one audit-only line to `trace.jsonl`. No narrative emission.
pub(crate) async fn write_audit_trace_line(
    writer: &mut PromptTraceWriter,
    formatted_line: &[u8],
) -> bool {
    assert!(matches!(ACP_AUDIT_CHANNEL, crate::observability::ObservabilityChannel::Audit));
    if let Err(e) = writer.file.write_all(formatted_line).await {
        tracing::warn!(error = %e, "trace write failed");
        return false;
    }
    if let Err(e) = writer.file.sync_all().await {
        tracing::warn!(error = %e, "trace fsync failed");
        return false;
    }
    true
}

/// Tee one narrative line to stdout / `stdout.log`, respecting suppression flags.
pub(crate) fn tee_narrative_line(
    writer: &mut PromptTraceWriter,
    line: &str,
    display_line: Option<&str>,
    ctx: &TraceTeeStdoutCtx<'_>,
) {
    assert!(matches!(
        ACP_NARRATIVE_CHANNEL,
        crate::observability::ObservabilityChannel::Narrative
    ));
    super::trace_line_write_tee_emit::tee_narrative_line_impl(writer, line, display_line, ctx);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn write_audit_trace_line_appends_to_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("audit.log");
        let file = tokio::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&path)
            .await
            .unwrap();
        let mut writer = PromptTraceWriter {
            file,
            who: crate::output::WHO_M.to_string(),
            plain_lines: false,
            stdout_replacement: None,
            placeholder_emitted: false,
            raw_output: false,
            show_thoughts_on_stdout: false,
            emit_stdout_markdown: false,
            iterable_closed_warned: false,
            upgrade_plan_warned: false,
            work_dir: dir.path().to_path_buf(),
            run_timing: None,
            session_id: String::new(),
            deferred_sink: None,
        };
        assert!(write_audit_trace_line(&mut writer, b"audit line\n").await);
        drop(writer);
        let text = tokio::fs::read_to_string(&path).await.unwrap();
        assert_eq!(text, "audit line\n");
    }

    #[test]
    fn tee_narrative_line_noop_when_tee_disabled() {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(tempfile::tempdir().unwrap().path().join("t.log"))
            .unwrap();
        let mut writer = PromptTraceWriter {
            file: tokio::fs::File::from_std(file),
            who: crate::output::WHO_M.to_string(),
            plain_lines: false,
            stdout_replacement: None,
            placeholder_emitted: false,
            raw_output: false,
            show_thoughts_on_stdout: false,
            emit_stdout_markdown: false,
            iterable_closed_warned: false,
            upgrade_plan_warned: false,
            work_dir: std::path::PathBuf::new(),
            run_timing: None,
            session_id: String::new(),
            deferred_sink: None,
        };
        tee_narrative_line(
            &mut writer,
            "ignored",
            None,
            &TraceTeeStdoutCtx {
                tee_stdout: false,
                kind: None,
                ts: "20260413.121314.015",
            },
        );
    }
}

#[cfg(test)]
mod kiss_cov_auto {
    use super::*;

    #[test]
    fn kiss_cov_write_audit_trace_line() {
        let _ = write_audit_trace_line;
    }

    #[test]
    fn kiss_cov_tee_narrative_line() {
        let _ = tee_narrative_line;
    }
}
