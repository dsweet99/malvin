use super::acp_trace_shim::{emit_mini_http_exchange, trace_for_run_dir, MiniHttpExchangeRecord};
use super::trace::{record_http_exchange, MiniTraceSink};
use crate::agent_backend::test_support::test_io;

#[test]
fn record_http_exchange_writes_and_noops() {
    use super::acp_trace_shim::MiniHttpExchangeRecord;
    let noop_sink = MiniTraceSink::new(None, test_io());
    record_http_exchange(
        &noop_sink,
        MiniHttpExchangeRecord {
            attempt: 1,
            status: None,
            body: None,
            error: None,
        },
    );
    let tmp = tempfile::tempdir().expect("tempdir");
    let sink = MiniTraceSink::new(Some(tmp.path().to_path_buf()), test_io());
    record_http_exchange(
        &sink,
        MiniHttpExchangeRecord {
            attempt: 2,
            status: Some(200),
            body: Some("payload"),
            error: Some("err".into()),
        },
    );
    let text = std::fs::read_to_string(tmp.path().join("trace.jsonl")).expect("trace");
    assert!(text.contains("miniHttpExchange"));
    let _ = record_http_exchange;
}

#[test]
fn emit_mini_http_exchange_keeps_small_bodies_intact() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let trace = trace_for_run_dir(tmp.path());
    let small = "short body";
    emit_mini_http_exchange(
        &trace,
        MiniHttpExchangeRecord {
            attempt: 1,
            status: Some(200),
            body: Some(small),
            error: None,
        },
    );
    let text = std::fs::read_to_string(tmp.path().join("trace.jsonl")).expect("trace");
    assert!(text.contains(small));
    assert!(!text.contains("truncated"));
}

#[test]
fn emit_mini_http_exchange_records_success_and_failure_shapes() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let trace = trace_for_run_dir(tmp.path());
    emit_mini_http_exchange(
        &trace,
        MiniHttpExchangeRecord {
            attempt: 1,
            status: Some(200),
            body: Some("{\"ok\":true}"),
            error: None,
        },
    );
    emit_mini_http_exchange(
        &trace,
        MiniHttpExchangeRecord {
            attempt: 2,
            status: None,
            body: None,
            error: Some("transport reset".into()),
        },
    );
    let text = std::fs::read_to_string(tmp.path().join("trace.jsonl")).expect("trace");
    assert!(text.contains("\"status\":200"));
    assert!(text.contains("transport reset"));
}

#[test]
fn emit_mini_http_exchange_truncates_large_bodies_in_trace() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let trace = trace_for_run_dir(tmp.path());
    let huge = "x".repeat(70 * 1024);
    emit_mini_http_exchange(
        &trace,
        MiniHttpExchangeRecord {
            attempt: 2,
            status: Some(200),
            body: Some(&huge),
            error: None,
        },
    );
    let text = std::fs::read_to_string(tmp.path().join("trace.jsonl")).expect("trace");
    assert!(text.contains("miniHttpExchange"));
    assert!(text.contains("truncated"));
    assert!(!text.contains(&huge));
}
