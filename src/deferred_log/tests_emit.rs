use std::path::PathBuf;
use std::time::Duration;

use crate::deferred_log::config::{
    defer_log_cursor_dir_from_env, defer_log_enabled_from_env, defer_log_max_age_from_env,
    defer_log_max_drain_from_env, env_is_zero, DeferredLogConfig,
};
use crate::deferred_log::emit::emit_deferred_entry;
use crate::deferred_log::enrich::{enriched_tool_plain, styled_tool_payload, synthetic_tool_done};
use crate::deferred_log::sink_build::build_display_log_entry;
use crate::deferred_log::test_fixtures::{
    capture_stdout_log, capture_stdout_render, test_tool_entry, zero_age_defer_shared,
    SharedDeferSink,
};
use crate::deferred_log::tool_enrich::tool_drain_enrich_fields;
use crate::deferred_log::{
    build_acp_tee_entry, build_raw_line_entry, install_stdout_hooks, register_active_sink,
    unregister_active_sink, AcpTeeBuild, TeeSinkMeta,
};
use std::sync::Arc;
use crate::deferred_log::types::{DeferredPayload, ToolDrainMeta};

#[test]
fn emit_display_log_entry_writes_timestamped_log_line() {
    let display = "[malvin.........] deferred-hb".to_string();
    let log = "20260524.000000.000 [malvin.........] deferred-hb".to_string();
    let (terminal, text) = capture_stdout_render(|| {
        emit_deferred_entry(&build_display_log_entry(display.clone(), log.clone()));
    });
    let line = text.lines().next().expect("log line");
    assert_eq!(line, log);
    assert_eq!(terminal.trim(), display);
    assert!(crate::output::is_log_timestamp_token(
        line.split_whitespace().next().expect("timestamp"),
    ));
    assert!(!display.starts_with("20"));
}

#[test]
#[allow(unsafe_code)]
fn defer_config_env_helpers_return_values() {
    unsafe {
        std::env::set_var("MALVIN_DEFER_LOG", "false");
    }
    assert!(env_is_zero("MALVIN_DEFER_LOG"));
    unsafe {
        std::env::remove_var("MALVIN_DEFER_LOG");
        std::env::remove_var("MALVIN_TEST_NO_REAL_AGENT");
    }
    assert!(defer_log_enabled_from_env());
    unsafe {
        std::env::set_var("MALVIN_TEST_NO_REAL_AGENT", "1");
    }
    assert!(!defer_log_enabled_from_env());
    unsafe {
        std::env::remove_var("MALVIN_TEST_NO_REAL_AGENT");
        std::env::set_var("MALVIN_DEFER_LOG_MAX_DRAIN", "128");
    }
    assert_eq!(defer_log_max_drain_from_env(), 128);
    unsafe {
        std::env::remove_var("MALVIN_DEFER_LOG_MAX_DRAIN");
    }
    assert!(defer_log_max_age_from_env() >= Duration::from_millis(0));
    let cfg = DeferredLogConfig::from_env();
    assert!(cfg.max_drain_per_log >= 1);
    assert!(!defer_log_cursor_dir_from_env().as_os_str().is_empty());
}

#[test]
fn styled_tool_payload_formats_plain_and_markdown() {
    use crate::cursor_store::ToolCallArgs;
    let (plain, display) = styled_tool_payload("Read file · 1ms", false);
    assert_eq!(plain, "Read file · 1ms");
    assert!(!display.is_empty());
    let (md_plain, _md_display) = styled_tool_payload("Read file · 1ms", true);
    assert_eq!(md_plain, "Read file · 1ms");
    assert!(!md_plain.starts_with('['));
    let meta = ToolDrainMeta {
        tool_call_id: "t1".to_string(),
        kind: "read".to_string(),
        elapsed: Duration::from_millis(3),
        raw_output: None,
        fallback_plain: "Read file · 3ms".to_string(),
    };
    let args = ToolCallArgs {
        path: Some("/tmp/x.rs".to_string()),
        line_range: None,
    };
    let parsed = synthetic_tool_done(&meta, &args);
    assert_eq!(parsed.kind, "read");
    let tmp = tempfile::tempdir().unwrap();
    let (enriched, _) = enriched_tool_plain(&meta, Some(&args), tmp.path(), false);
    assert!(enriched.contains("x.rs") || enriched.contains("Read"));
}

fn push_acp_tee_marker(shared: &SharedDeferSink, i: usize, marker: &str) {
    shared
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push_entry(build_acp_tee_entry(AcpTeeBuild {
            tee: TeeSinkMeta {
                who: "kpop".to_string(),
                ts: format!("20260525.102620.{i:03}"),
                emit_stdout_markdown: false,
            },
            kind: None,
            line: marker.to_string(),
            display: None,
            dim_payload: false,
        }));
}

fn assert_fifo_markers(text: &str, markers: &[&str]) {
    let pos_a = text.find(markers[0]).expect("marker A");
    let pos_b = text.find(markers[1]).expect("marker B");
    let pos_c = text.find(markers[2]).expect("marker C");
    assert!(
        pos_a < pos_b && pos_b < pos_c,
        "ACP tee FIFO emit order violated with active sink: {text:?}"
    );
}

fn assert_monotonic_log_timestamps(text: &str, prefix: &str) {
    let stamps: Vec<&str> = text
        .lines()
        .filter_map(|l| l.split_whitespace().next())
        .filter(|t| t.starts_with(prefix))
        .collect();
    let inversions = stamps.windows(2).filter(|w| w[0] > w[1]).count();
    assert_eq!(inversions, 0, "log timestamps must be monotonic; got {stamps:?}");
}

#[test]
fn fifo_emit_acp_tee_order_with_active_sink() {
    let markers = ["FIFO_ACP_A", "FIFO_ACP_B", "FIFO_ACP_C"];
    let text = capture_stdout_log(|| {
        let shared = zero_age_defer_shared("fifo-acp-tee");
        register_active_sink(Arc::clone(&shared));
        install_stdout_hooks();
        for (i, marker) in markers.iter().enumerate() {
            push_acp_tee_marker(&shared, i, marker);
        }
        unregister_active_sink();
    });
    assert_fifo_markers(&text, &markers);
    assert_monotonic_log_timestamps(&text, "20260525");
}

#[test]
fn raw_line_emit_flushes_without_redefer() {
    let text = capture_stdout_log(|| {
        emit_deferred_entry(&build_raw_line_entry(
            "RAW_EMIT_PROBE".to_string(),
            "kpop".to_string(),
            "20260525.102620.100".to_string(),
        ));
    });
    assert!(text.contains("RAW_EMIT_PROBE"));
}

#[test]
fn tool_summary_emit_builds_acp_event() {
    let text = capture_stdout_log(|| {
        emit_deferred_entry(&test_tool_entry("Read file · 2ms"));
    });
    assert!(text.contains("Read file"));
}

#[test]
fn emit_and_build_helpers_cover_entry_shapes() {
    let entry = build_display_log_entry("d".to_string(), "l".to_string());
    let DeferredPayload::DisplayLog { display, log } = entry.payload else {
        panic!("expected display log payload");
    };
    assert_eq!(display, "d");
    assert_eq!(log, "l");
    let acp = build_acp_tee_entry(AcpTeeBuild {
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
    assert!(matches!(acp.payload, DeferredPayload::AcpTee { .. }));
    let raw = build_raw_line_entry("raw".to_string(), "w".to_string(), "ts".to_string());
    assert!(matches!(raw.payload, DeferredPayload::RawLine { .. }));
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
    let (enrich, meta) = tool_drain_enrich_fields(&parsed, &tracker, "[Read file · 1ms]");
    assert!(enrich.is_some());
    assert_eq!(meta.expect("meta").kind, "read");
    tracker.set_work_dir(PathBuf::from("/tmp"));
    let (enrich_with_dir, _) = tool_drain_enrich_fields(&parsed, &tracker, "[Read file · 1ms]");
    assert!(enrich_with_dir.is_some());
}
