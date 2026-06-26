//! Shared parsers for mini vs ACP observability parity tests.

use std::path::Path;

pub fn assert_acp_trace_schema(path: &Path) {
    let text = std::fs::read_to_string(path).expect("read trace.jsonl");
    assert!(!text.is_empty(), "trace.jsonl must not be empty");
    let mut direction_count = 0;
    for line in text.lines().filter(|l| !l.is_empty()) {
        let record: serde_json::Value =
            serde_json::from_str(line).expect("valid jsonl in trace.jsonl");
        assert!(
            record.get("direction").is_some() || record.get("message").is_some(),
            "each trace line must have direction or message: {record}"
        );
        if record.get("direction").is_some() {
            direction_count += 1;
        }
    }
    assert!(direction_count > 0, "trace must contain direction lines");
}

pub fn assert_stdout_tool_vocab(path: &Path, expected_kinds: &[&str]) {
    let text = std::fs::read_to_string(path).expect("read stdout.log");
    for kind in expected_kinds {
        assert!(
            text.contains(&format!("{kind} ")),
            "stdout.log must contain tool summary kind {kind:?}; got:\n{text}"
        );
    }
}

pub fn assert_prompts_contains(path: &Path, substring: &str) {
    let text = std::fs::read_to_string(path).expect("read prompts.log");
    assert!(
        text.contains(substring),
        "prompts.log must contain {substring:?}; got:\n{text}"
    );
}

pub fn trace_contains_substring(path: &Path, needle: &str) {
    let text = std::fs::read_to_string(path).expect("read trace");
    assert!(
        text.contains(needle),
        "trace.jsonl must contain {needle:?}"
    );
}

pub fn stdout_m_before_t_on_multiturn(path: &Path) {
    let text = std::fs::read_to_string(path).expect("read stdout");
    let m_pos = text.find("m|").expect("stdout must contain m|");
    let t_pos = text.find("t|").expect("stdout must contain t|");
    assert!(m_pos < t_pos, "m| must appear before t| on multi-turn run");
}
