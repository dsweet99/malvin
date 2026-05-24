use serde_json::{Value, json};
use std::io::Write;
use std::path::PathBuf;

static TRACE_JSONL_WRITE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[derive(Clone)]
pub struct AcpJsonlTrace {
    pub path: PathBuf,
    pub name: String,
}

impl AcpJsonlTrace {
    #[must_use]
    pub const fn new(path: PathBuf, name: String) -> Self {
        Self { path, name }
    }

    pub fn append_line(&self, direction: &str, line: &str) {
        let now = chrono::Local::now();
        let ts = format!(
            "{}.{:03}",
            now.format("%Y%m%d.%H%M%S"),
            now.timestamp_subsec_millis()
        );
        let mut record = json!({
            "ts": ts,
            "name": self.name,
            "direction": direction
        });
        if let Some(obj) = record.as_object_mut() {
            if let Ok(msg) = serde_json::from_str::<Value>(line) {
                obj.insert("message".to_string(), msg);
            } else {
                obj.insert("raw".to_string(), Value::String(line.to_string()));
            }
        }
        let line = record.to_string();
        let _guard = TRACE_JSONL_WRITE_LOCK.lock().ok();
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .and_then(|mut f| writeln!(f, "{line}"));
    }
}

#[cfg(test)]
mod tests {
    use super::AcpJsonlTrace;

    #[test]
    fn trace_jsonl_records_named_two_way_lines() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("trace.jsonl");
        let trace = AcpJsonlTrace::new(path.clone(), "Ab3x9".to_string());
        trace.append_line("out", r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#);
        trace.append_line("in", r#"{"jsonrpc":"2.0","id":1,"result":{}}"#);
        let text = std::fs::read_to_string(path).expect("read trace");
        let records: Vec<serde_json::Value> = text
            .lines()
            .map(|line| serde_json::from_str(line).expect("valid jsonl record"))
            .collect();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0]["name"], "Ab3x9");
        assert_eq!(records[0]["direction"], "out");
        assert_eq!(records[0]["message"]["method"], "initialize");
        assert_eq!(records[1]["name"], "Ab3x9");
        assert_eq!(records[1]["direction"], "in");
        assert_eq!(records[1]["message"]["result"], serde_json::json!({}));
    }

    #[test]
    fn trace_jsonl_append_records_raw_line_before_human_summary() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("trace.jsonl");
        let trace = AcpJsonlTrace::new(path.clone(), "kpop".to_string());
        let raw = r#"{"jsonrpc":"2.0","method":"session/update","params":{"update":{"sessionUpdate":"tool_call","toolCallId":"tool_x","kind":"read","status":"pending","title":"Read"}}}"#;
        trace.append_line("in", raw);
        let text = std::fs::read_to_string(path).expect("read");
        assert!(text.contains("tool_call"));
        assert!(text.contains("tool_x"));
        assert!(!text.contains("[tool]"));
    }
}
