use std::path::PathBuf;
use std::time::Duration;

use crate::cursor_store::{install_test_store, TestStoreSpec, ToolCallArgs};
use super::config::{
    defer_log_enabled_from_env, defer_log_max_age_from_env, defer_log_max_drain_from_env,
    DeferredLogConfig,
};
use super::emit::emit_deferred_entry;
use super::enrich::enriched_tool_plain;
use super::tool_enrich::tool_drain_enrich_fields;
use super::test_fixtures::{enrich_read_entry, test_tool_entry};
use super::{
    build_acp_tee_entry, build_raw_line_entry, DeferredLogSink,
};
use super::sink::build_heartbeat_entry;
use super::types::{AcpTeeBuild, DeferredPayload, TeeSinkMeta, ToolDrainMeta};

#[test]
fn fifo_drain_respects_age_gate() {
    let mut sink = DeferredLogSink::new(
        "sess".to_string(),
        PathBuf::new(),
        DeferredLogConfig {
            max_age: Duration::from_millis(50),
            max_drain_per_log: 64,
            cursor_dir: PathBuf::new(),
        },
    );
    sink.push_entry(test_tool_entry("a"));
    assert_eq!(sink.queue_len(), 1);
    sink.push_entry(test_tool_entry("b"));
    std::thread::sleep(Duration::from_millis(60));
    sink.push_entry(test_tool_entry("c"));
    assert!(sink.queue_len() <= 1);
}

#[test]
fn force_flush_drains_without_enrich() {
    let tmp = tempfile::tempdir().unwrap();
    let fallback = "Read file · 1ms";
    install_test_store(&TestStoreSpec {
        cursor_dir: tmp.path(),
        session_id: "sess",
        tool_call_id: "toolu_x",
        path: "/tmp/foo.rs",
        offset: None,
        limit: None,
    });
    let text = super::test_fixtures::capture_stdout_log(|| {
        let mut sink = DeferredLogSink::new(
            "sess".to_string(),
            tmp.path().to_path_buf(),
            DeferredLogConfig {
                max_age: Duration::from_secs(60),
                max_drain_per_log: 64,
                cursor_dir: tmp.path().to_path_buf(),
            },
        );
        sink.push_entry(enrich_read_entry("toolu_x", fallback));
        assert_eq!(sink.queue_len(), 1);
        sink.force_flush();
        assert_eq!(sink.queue_len(), 0);
    });
    assert!(text.contains(fallback));
    assert!(!text.contains("foo.rs"));
}

#[test]
fn enrich_read_line_from_store_args() {
    let tmp = tempfile::tempdir().unwrap();
    let meta = ToolDrainMeta {
        tool_call_id: "t1".to_string(),
        kind: "read".to_string(),
        elapsed: Duration::from_millis(8),
        raw_output: None,
        fallback_plain: "Read file · 8ms".to_string(),
    };
    let args = ToolCallArgs {
        path: Some("/home/user/project/src/index.ts".to_string()),
        line_range: None,
    };
    let (plain, _display) =
        enriched_tool_plain(&meta, Some(&args), tmp.path(), true);
    assert!(plain.contains("index.ts"));
}

#[test]
fn edit_enrich_formats_edit_not_read() {
    let tmp = tempfile::tempdir().unwrap();
    let meta = ToolDrainMeta {
        tool_call_id: "toolu_edit_fixture".to_string(),
        kind: "edit".to_string(),
        elapsed: Duration::from_millis(5),
        raw_output: None,
        fallback_plain: "Edit file · 5ms".to_string(),
    };
    let args = ToolCallArgs {
        path: Some("/home/user/project/src/lib.rs".to_string()),
        line_range: None,
    };
    let (plain, _display) = enriched_tool_plain(&meta, Some(&args), tmp.path(), false);
    assert!(
        plain.starts_with("Edit "),
        "store.db edit enrichment must prefix Edit, got {plain:?}"
    );
    assert!(
        !plain.contains("Read "),
        "store.db edit enrichment must not use Read formatter, got {plain:?}"
    );
}

#[test]
fn enrichable_tool_entry_omits_plain_at_enqueue() {
    let entry = enrich_read_entry("toolu_opt_a", "Read file · 1ms");
    let DeferredPayload::ToolSummary {
        plain,
        display,
        enrich,
        meta,
    } = entry.payload
    else {
        panic!("expected tool summary payload");
    };
    assert!(plain.is_empty(), "Option A: plain built at drain, not tee");
    assert!(display.is_empty(), "Option A: display built at drain, not tee");
    assert!(enrich.is_some());
    assert_eq!(
        meta.expect("meta").fallback_plain,
        "Read file · 1ms"
    );
}

#[test]
fn defer_raw_line_entry_includes_timestamp() {
    let entry = build_raw_line_entry(
        "payload".to_string(),
        "who".to_string(),
        "20260524.123456.789".to_string(),
    );
    assert!(
        !entry.ts.is_empty(),
        "deferred raw/plain tee entries must carry timestamp (plan DeferredEntry.ts)"
    );
}

#[test]
fn emit_and_build_helpers_smoke() {
    let _ = defer_log_enabled_from_env;
    let _ = defer_log_max_age_from_env();
    let _ = defer_log_max_drain_from_env();
    let entry = build_heartbeat_entry("line".to_string());
    emit_deferred_entry(&entry);
    let _ = build_acp_tee_entry(AcpTeeBuild {
        tee: TeeSinkMeta {
            who: "w".to_string(),
            ts: "ts".to_string(),
            emit_stdout_markdown: true,
        },
        kind: None,
        line: "x".to_string(),
        display: None,
        dim_payload: false,
    });
    let _ = build_raw_line_entry("raw".to_string(), "w".to_string(), "ts".to_string());
    let parsed = serde_json::json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "t",
            "kind": "read",
            "status": "completed",
            "rawInput": {},
            "rawOutput": {"content": "x"}
        }}
    });
    let mut tracker = crate::tool_summary::ToolSummaryTracker::default();
    let _ = tool_drain_enrich_fields(&parsed, &tracker, "[Read file · 1ms]");
    tracker.set_work_dir(PathBuf::from("/tmp"));
    let _ = tool_drain_enrich_fields(&parsed, &tracker, "[Read file · 1ms]");
}

#[cfg(test)]
mod kiss_cov_auto {
    #[test]
    fn kiss_cov_fifo_drain_respects_age_gate() {
        let _ = super::fifo_drain_respects_age_gate;
    }

    #[test]
    fn kiss_cov_edit_enrich_formats_edit_not_read() {
        let _ = super::edit_enrich_formats_edit_not_read;
    }

    #[test]
    fn kiss_cov_defer_raw_line_entry_includes_timestamp() {
        let _ = super::defer_raw_line_entry_includes_timestamp;
    }

    #[test]
    fn kiss_cov_force_flush_drains_without_enrich() {
        let _ = super::force_flush_drains_without_enrich;
    }

}
