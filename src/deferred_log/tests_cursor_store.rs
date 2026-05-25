use crate::cursor_store::{
    find_store_path, install_test_store, CursorStoreCache, TestStoreSpec,
};

#[test]
fn ingest_prepare_failure_disables_cache_and_stops_retry() {
    let tmp = tempfile::tempdir().unwrap();
    let session_dir = tmp
        .path()
        .join("acp-sessions")
        .join("schema-mismatch");
    std::fs::create_dir_all(&session_dir).unwrap();
    let db_path = session_dir.join("store.db");
    let conn = rusqlite::Connection::open(&db_path).expect("open store.db");
    conn.execute_batch("CREATE TABLE other (x INTEGER);")
        .expect("schema without blobs");
    drop(conn);
    let mut cache = CursorStoreCache::new("schema-mismatch".to_string(), tmp.path().to_path_buf());
    cache.ensure_open();
    assert!(cache.store_path().is_some());
    cache.ingest_new_blobs();
    cache.ingest_new_blobs();
    assert!(
        cache.ingest_calls <= 1,
        "prepare failure must disable ingest for the session (got {} calls)",
        cache.ingest_calls
    );
}

#[test]
fn cursor_store_second_ingest_is_noop_without_new_rows() {
    let tmp = tempfile::tempdir().unwrap();
    install_test_store(&TestStoreSpec {
        cursor_dir: tmp.path(),
        session_id: "sess",
        tool_call_id: "toolu_abc",
        path: "/proj/a.rs",
        offset: None,
        limit: None,
    });
    let mut cache = CursorStoreCache::new("sess".to_string(), tmp.path().to_path_buf());
    cache.ensure_open();
    cache.ingest_new_blobs();
    assert_eq!(cache.get("toolu_abc").expect("args").path.as_deref(), Some("/proj/a.rs"));
    assert_eq!(cache.ingest_calls, 1);
    cache.ingest_new_blobs();
    assert_eq!(cache.ingest_calls, 2);
    assert_eq!(cache.get("toolu_abc").expect("args").path.as_deref(), Some("/proj/a.rs"));
}

#[test]
fn cursor_store_ingests_tool_call_path() {
    let tmp = tempfile::tempdir().unwrap();
    install_test_store(&TestStoreSpec {
        cursor_dir: tmp.path(),
        session_id: "sess",
        tool_call_id: "toolu_abc",
        path: "/proj/a.rs",
        offset: None,
        limit: None,
    });
    let mut cache = CursorStoreCache::new("sess".to_string(), tmp.path().to_path_buf());
    cache.ensure_open();
    cache.ingest_new_blobs();
    let args = cache.get("toolu_abc").expect("args");
    assert_eq!(args.path.as_deref(), Some("/proj/a.rs"));
    assert!(cache.store_path().is_some());
    cache.ingest_new_blobs();
}

#[test]
#[ignore = "manual: uses live ~/.cursor store.db from defer_enrich e2e"]
fn live_cursor_store_ingests_read_path() {
    let session_id = std::env::var("MALVIN_LIVE_SESSION_ID")
        .unwrap_or_else(|_| "06c114e6-7f81-4763-ae05-f4c1ae2f9e09".to_string());
    let tool_id = std::env::var("MALVIN_LIVE_TOOL_ID")
        .unwrap_or_else(|_| "tool_994f8814-1c09-4fc0-9dad-513c39f3ca8".to_string());
    let home = crate::user_home_dir();
    let mut cache = CursorStoreCache::new(session_id, home.join(".cursor"));
    cache.ensure_open();
    assert!(cache.store_path().is_some(), "store.db must exist");
    cache.ingest_new_blobs();
    let path = cache
        .get(&tool_id)
        .and_then(|a| a.path.clone())
        .unwrap_or_else(|| panic!("missing tool-call {tool_id} in store cache"));
    assert!(
        path.contains("defer_enrich_probe"),
        "expected read path in cache, got {path:?}"
    );
}

#[test]
fn legacy_store_path_discovery() {
    let tmp = tempfile::tempdir().unwrap();
    let legacy = tmp.path().join("chats").join("abc123").join("legacy-sess");
    std::fs::create_dir_all(&legacy).unwrap();
    std::fs::write(legacy.join("store.db"), b"").unwrap();
    assert_eq!(
        find_store_path(tmp.path(), "legacy-sess"),
        Some(legacy.join("store.db"))
    );
}
