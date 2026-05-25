use super::{active_mutex, is_registered, register, unregister, SharedDeferSink, try_log, try_push};
use crate::deferred_log::{
    build_heartbeat_entry, build_tagged_stdout_entry, DeferredLogSink,
};
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn try_push_queues_heartbeat_when_defer_sink_mutex_held() {
    let shared: SharedDeferSink = Arc::new(std::sync::Mutex::new(DeferredLogSink::new(
        "contention".to_string(),
        PathBuf::new(),
        crate::deferred_log::config::DeferredLogConfig {
            max_age: std::time::Duration::from_secs(3600),
            max_drain_per_log: 64,
            cursor_dir: PathBuf::new(),
        },
    )));
    register(Arc::clone(&shared));
    let hb = build_heartbeat_entry("20260524.000000.000 [malvin........] heartbeat".into());
    {
        let _acp_hold = shared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(
            try_push(hb),
            "wall-clock defer hook must not drop heartbeat when ACP path holds the sink mutex"
        );
        assert_eq!(super::pending_len(), 1, "heartbeat must sit in pending until mutex released");
    }
    shared
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push_entry(build_tagged_stdout_entry("flush".into(), "flush".into()));
    assert_eq!(super::pending_len(), 0);
    assert!(
        shared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .queue_len()
            >= 1,
        "heartbeat must reach defer sink queue after flush"
    );
    unregister();
}

#[test]
fn pending_flush_covers_queue_and_drain_paths() {
    let _ = super::pending_entries;
    let _ = super::queue_pending;
    let shared: SharedDeferSink = Arc::new(std::sync::Mutex::new(DeferredLogSink::new(
        "pending".to_string(),
        PathBuf::new(),
        crate::deferred_log::config::DeferredLogConfig::from_env(),
    )));
    super::queue_pending(build_heartbeat_entry("pending-hb".into()));
    assert_eq!(super::pending_len(), 1);
    super::flush_pending_into(
        &mut shared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
    );
    assert_eq!(super::pending_len(), 0);
    unregister();
}

#[test]
fn active_slot_register_unregister_roundtrip() {
    let _ = active_mutex;
    let shared: SharedDeferSink = Arc::new(std::sync::Mutex::new(DeferredLogSink::new(
        "active".to_string(),
        PathBuf::new(),
        crate::deferred_log::config::DeferredLogConfig::from_env(),
    )));
    register(Arc::clone(&shared));
    assert!(is_registered());
    assert!(try_log(build_tagged_stdout_entry("d".into(), "l".into())));
    assert!(try_push(build_heartbeat_entry("hb".into())));
    unregister();
    assert!(!is_registered());
    assert!(!try_log(build_tagged_stdout_entry("d".into(), "l".into())));
    register(Arc::clone(&shared));
    unregister();
}
