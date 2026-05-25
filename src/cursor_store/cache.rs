use std::collections::HashMap;
use std::path::PathBuf;

use rusqlite::{Connection, OpenFlags};

use super::parse::parse_tool_call_args_from_blob;
use super::path::find_store_path;
use super::types::ToolCallArgs;

pub struct CursorStoreCache {
    session_id: String,
    cursor_dir: PathBuf,
    conn: Option<Connection>,
    last_rowid: i64,
    map: HashMap<String, ToolCallArgs>,
    disabled: bool,
    #[cfg(test)]
    pub(crate) ingest_calls: usize,
}

impl CursorStoreCache {
    pub fn new(session_id: String, cursor_dir: PathBuf) -> Self {
        Self {
            session_id,
            cursor_dir,
            conn: None,
            last_rowid: 0,
            map: HashMap::new(),
            disabled: false,
            #[cfg(test)]
            ingest_calls: 0,
        }
    }

    pub fn ensure_open(&mut self) {
        if self.conn.is_some() || self.disabled {
            return;
        }
        let Some(path) = self.store_path() else {
            tracing::debug!(session_id = %self.session_id, "cursor store path missing");
            self.disabled = true;
            return;
        };
        let Ok(conn) = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY) else {
            tracing::debug!(path = %path.display(), "cursor store open failed");
            self.disabled = true;
            return;
        };
        tracing::debug!(path = %path.display(), "cursor store opened");
        self.conn = Some(conn);
    }

    pub fn ingest_new_blobs(&mut self) {
        if self.disabled {
            return;
        }
        #[cfg(test)]
        {
            self.ingest_calls += 1;
        }
        let Some(conn) = self.conn.as_ref() else {
            return;
        };
        let Ok(mut stmt) = conn.prepare("SELECT rowid, data FROM blobs WHERE rowid > ?") else {
            self.disabled = true;
            return;
        };
        let rows = stmt.query_map([self.last_rowid], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        });
        let Ok(rows) = rows else {
            self.disabled = true;
            return;
        };
        for row in rows.flatten() {
            self.last_rowid = row.0;
            for (id, args) in parse_tool_call_args_from_blob(&row.1) {
                self.map.insert(id, args);
            }
        }
    }

    pub fn get(&self, tool_call_id: &str) -> Option<&ToolCallArgs> {
        self.map.get(tool_call_id)
    }

    pub fn store_path(&self) -> Option<PathBuf> {
        find_store_path(&self.cursor_dir, &self.session_id)
    }
}

#[cfg(test)]
pub struct TestStoreSpec<'a> {
    pub cursor_dir: &'a std::path::Path,
    pub session_id: &'a str,
    pub tool_call_id: &'a str,
    pub path: &'a str,
    pub offset: Option<u64>,
    pub limit: Option<u64>,
}

#[cfg(test)]
pub fn install_test_store(spec: &TestStoreSpec<'_>) -> PathBuf {
    let session_dir = spec
        .cursor_dir
        .join("acp-sessions")
        .join(spec.session_id);
    std::fs::create_dir_all(&session_dir).expect("session dir");
    let db_path = session_dir.join("store.db");
    let conn = rusqlite::Connection::open(&db_path).expect("open store.db");
    conn.execute_batch(
        "CREATE TABLE blobs (id TEXT PRIMARY KEY, data BLOB);
         CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT);",
    )
    .expect("schema");
    let mut args = serde_json::json!({ "path": spec.path });
    if let (Some(offset), Some(limit)) = (spec.offset, spec.limit) {
        args["offset"] = serde_json::json!(offset);
        args["limit"] = serde_json::json!(limit);
    }
    let blob = serde_json::json!({
        "role": "assistant",
        "content": [{
            "type": "tool-call",
            "toolCallId": spec.tool_call_id,
            "toolName": "Read",
            "args": args
        }]
    });
    let blob_id = if spec.offset.is_some() {
        "blob-range-001"
    } else {
        "blob-assistant-001"
    };
    conn.execute(
        "INSERT INTO blobs (id, data) VALUES (?1, ?2)",
        rusqlite::params![blob_id, blob.to_string()],
    )
    .expect("insert blob");
    db_path
}
