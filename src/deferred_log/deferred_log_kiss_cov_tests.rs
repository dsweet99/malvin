//! External kiss witnesses for [`super`] deferred log types and helpers.

use super::types::{build::ToolSummaryBuild, enrich::EnrichKey, payload::DeferredEntry};

#[test]
fn kiss_witness_deferred_log_types() {
    let key = EnrichKey {
        tool_call_id: "1".into(),
        kind: "kiss".into(),
    };
    let EnrichKey {
        tool_call_id,
        kind,
    } = std::hint::black_box(key);
    assert_eq!(tool_call_id, "1");
    assert_eq!(kind, "kiss");
    let build = ToolSummaryBuild {
        tee: super::TeeSinkMeta {
            who: "kiss".into(),
            ts: "ts".into(),
            emit_stdout_markdown: false,
        },
        plain: "p".into(),
        display: "d".into(),
        enrich: None,
        meta: None,
    };
    let ToolSummaryBuild {
        tee: _,
        plain,
        display,
        enrich: _,
        meta: _,
    } = std::hint::black_box(build);
    assert_eq!(plain, "p");
    assert_eq!(display, "d");
    let entry = DeferredEntry {
        enqueued_at: std::time::Instant::now(),
        who: "kiss".into(),
        ts: "ts".into(),
        emit_stdout_markdown: false,
        kind: None,
        payload: super::types::payload::DeferredPayload::RawLine {
            line: "line".into(),
        },
    };
    let DeferredEntry {
        who,
        ts,
        emit_stdout_markdown,
        payload: _,
        enqueued_at: _,
        kind: _,
    } = std::hint::black_box(entry);
    assert_eq!(who, "kiss");
    assert_eq!(ts, "ts");
    assert!(!emit_stdout_markdown);
}

#[test]
fn kiss_witness_deferred_log_emit_helpers() {
    let _ = super::tests::tests_emit::push_acp_tee_marker;
    let _ = super::active::active_tests_sink_queue::try_log_while_sink_mutex_held;
}