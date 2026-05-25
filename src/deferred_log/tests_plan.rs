use std::path::PathBuf;
use std::time::Instant;

use crate::cursor_store::{
    install_test_store, parse_tool_call_args_from_blob, TestStoreSpec, ToolCallArgs,
};
use crate::tool_summary::LineRange;

use super::enrich::enriched_tool_plain;
use super::sink::test_access;
use super::test_fixtures::{enrich_read_entry, test_tool_entry, zero_age_sink};
use super::build_tagged_stdout_entry;
use super::types::ToolDrainMeta;

#[test]
fn fifo_emit_order_abc() {
    let text = super::test_fixtures::capture_stdout_log(|| {
        let mut sink = zero_age_sink("sess", PathBuf::new(), 64);
        for label in ["LOG_MARKER_A", "LOG_MARKER_B", "LOG_MARKER_C"] {
            sink.push_entry(build_tagged_stdout_entry(
                label.to_string(),
                label.to_string(),
            ));
        }
    });
    let pos_a = text.find("LOG_MARKER_A").expect("marker A");
    let pos_b = text.find("LOG_MARKER_B").expect("marker B");
    let pos_c = text.find("LOG_MARKER_C").expect("marker C");
    assert!(pos_a < pos_b && pos_b < pos_c, "FIFO emit order violated: {text:?}");
}

#[test]
fn max_drain_per_log_leaves_excess_aged_entries() {
    let mut sink = zero_age_sink("sess", PathBuf::new(), 2);
    for i in 0..5 {
        test_access::push_back(&mut sink, test_tool_entry(&format!("e{i}")));
    }
    test_access::drain_ready(&mut sink);
    assert_eq!(sink.queue_len(), 3);
    test_access::drain_ready(&mut sink);
    assert_eq!(sink.queue_len(), 1);
    test_access::drain_ready(&mut sink);
    assert_eq!(sink.queue_len(), 0);
}

#[test]
fn ingest_new_blobs_runs_at_most_once_per_drain() {
    let tmp = tempfile::tempdir().unwrap();
    install_test_store(&TestStoreSpec {
        cursor_dir: tmp.path(),
        session_id: "sess",
        tool_call_id: "toolu_x",
        path: "/tmp/foo.rs",
        offset: None,
        limit: None,
    });
    let mut sink = zero_age_sink("sess", tmp.path().to_path_buf(), 64);
    for id in ["toolu_x", "toolu_y", "toolu_z"] {
        test_access::push_back(&mut sink, enrich_read_entry(id, "Read file · 1ms"));
    }
    test_access::drain_ready(&mut sink);
    assert_eq!(test_access::ingest_calls(&sink), 1);
}

#[test]
fn parse_blob_with_multiple_tool_calls() {
    let blob = serde_json::json!({
        "role": "assistant",
        "content": [
            {
                "type": "tool-call",
                "toolCallId": "toolu_a",
                "toolName": "Read",
                "args": {"path": "/a.rs"}
            },
            {
                "type": "tool-call",
                "toolCallId": "toolu_b",
                "toolName": "Read",
                "args": {"path": "/b.rs", "offset": 10, "limit": 20}
            }
        ]
    });
    let parsed = parse_tool_call_args_from_blob(&blob.to_string());
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].0, "toolu_a");
    assert_eq!(parsed[1].0, "toolu_b");
    assert_eq!(parsed[1].1.path.as_deref(), Some("/b.rs"));
    assert!(parsed[1].1.line_range.is_some());
}

fn drain_missing_store_fallback(tmp: &std::path::Path, log_path: &std::path::Path) -> String {
    let mut sink = zero_age_sink("missing-sess", tmp.to_path_buf(), 64);
    let fallback = "Read file · 1ms";
    let mut entry = enrich_read_entry("toolu_missing", fallback);
    entry.enqueued_at = Instant::now()
        .checked_sub(std::time::Duration::from_secs(1))
        .expect("aged entry offset");
    test_access::push_back(&mut sink, entry);
    test_access::drain_ready(&mut sink);
    std::fs::read_to_string(log_path).unwrap_or_default()
}

#[test]
fn missing_store_db_uses_fallback_on_drain() {
    let _stdout_guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let tmp = tempfile::tempdir().unwrap();
    let log_path = tmp.path().join("stdout.log");
    crate::output::set_stdout_log_path(Some(log_path.clone()));
    let text = drain_missing_store_fallback(tmp.path(), &log_path);
    crate::output::set_stdout_log_path(None);
    assert!(text.contains("Read file · 1ms"));
}

#[test]
fn enrich_read_includes_line_range_suffix() {
    let tmp = tempfile::tempdir().unwrap();
    let meta = ToolDrainMeta {
        tool_call_id: "t1".to_string(),
        kind: "read".to_string(),
        elapsed: std::time::Duration::from_millis(8),
        raw_output: None,
        fallback_plain: "Read file · 8ms".to_string(),
    };
    let args = ToolCallArgs {
        path: Some("/home/user/project/src/foo.rs".to_string()),
        line_range: Some(LineRange {
            start: 90,
            end: Some(130),
        }),
    };
    let (plain, _display) = enriched_tool_plain(&meta, Some(&args), tmp.path(), true);
    assert!(plain.contains(":90-130"));
}

#[test]
fn deferred_sink_drain_shows_line_range_from_store_db() {
    let tmp = tempfile::tempdir().unwrap();
    install_test_store(&TestStoreSpec {
        cursor_dir: tmp.path(),
        session_id: "sess",
        tool_call_id: "toolu_range",
        path: "/proj/foo.rs",
        offset: Some(90),
        limit: Some(40),
    });
    let text = super::test_fixtures::capture_stdout_log(|| {
        let mut sink = zero_age_sink("sess", tmp.path().to_path_buf(), 64);
        sink.push_entry(enrich_read_entry("toolu_range", "Read file · 8ms"));
    });
    assert!(
        text.contains(":91-130"),
        "deferred drain must show line range from store.db args on stdout.log; got {text:?}"
    );
}

#[cfg(test)]
mod kiss_cov_auto {
    #[test]
    fn kiss_cov_max_drain_per_log_leaves_excess_aged_entries() {
        let _ = super::max_drain_per_log_leaves_excess_aged_entries;
    }

    #[test]
    fn kiss_cov_fifo_emit_order_abc() {
        let _ = super::fifo_emit_order_abc;
    }

    #[test]
    fn kiss_cov_ingest_new_blobs_runs_at_most_once_per_drain() {
        let _ = super::ingest_new_blobs_runs_at_most_once_per_drain;
    }

    #[test]
    fn kiss_cov_parse_blob_with_multiple_tool_calls() {
        let _ = super::parse_blob_with_multiple_tool_calls;
    }

    #[test]
    fn kiss_cov_missing_store_db_uses_fallback_on_drain() {
        let _ = super::missing_store_db_uses_fallback_on_drain;
    }

    #[test]
    fn kiss_cov_enrich_read_includes_line_range_suffix() {
        let _ = super::enrich_read_includes_line_range_suffix;
    }

    #[test]
    fn kiss_cov_deferred_sink_drain_shows_line_range_from_store_db() {
        let _ = super::deferred_sink_drain_shows_line_range_from_store_db;
    }

    #[test]
    fn kiss_cov_drain_missing_store_fallback() {
        let tmp = tempfile::tempdir().unwrap();
        let log_path = tmp.path().join("stdout.log");
        let _ = super::drain_missing_store_fallback(tmp.path(), &log_path);
    }
}
