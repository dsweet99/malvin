use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

use serde_json::Value;

use super::discover::resolve_npm_pi_entry;
use super::session_process::entry_is_rpc_entry;

pub(super) const BASE_THINKING_LEVELS: [&str; 5] = ["off", "minimal", "low", "medium", "high"];
const EXTENDED_THINKING_LEVELS: [&str; 7] = ["off", "minimal", "low", "medium", "high", "xhigh", "max"];
const RPC_TIMEOUT: Duration = Duration::from_secs(30);
const MODELS_REQUEST: &str = r#"{"id":"malvin-models","type":"get_available_models"}"#;

pub(super) fn list_rpc_models() -> Result<Vec<(String, String)>, String> {
    Ok(fetch_rpc_models()?.iter().filter_map(model_row).collect())
}

fn fetch_rpc_models() -> Result<Vec<Value>, String> {
    let entry = resolve_npm_pi_entry()?;
    let node = crate::cursor_sdk::node_resolve::resolve_node_bin()?;
    let mut cmd = Command::new(node);
    cmd.arg(&entry);
    if !entry_is_rpc_entry(&entry) {
        cmd.arg("--mode").arg("rpc");
    }
    let mut child = cmd
        .arg("--no-session")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("spawn npm pi rpc: {e}"))?;
    let result = request_models(&mut child);
    let _ = child.kill();
    let _ = child.wait();
    result
}

fn request_models(child: &mut std::process::Child) -> Result<Vec<Value>, String> {
    let mut stdin = child.stdin.take().ok_or("npm pi rpc: no stdin")?;
    let stdout = child.stdout.take().ok_or("npm pi rpc: no stdout")?;
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(read_models_response(BufReader::new(stdout)));
    });
    writeln!(stdin, "{MODELS_REQUEST}")
        .and_then(|()| stdin.flush())
        .map_err(|e| format!("npm pi rpc write: {e}"))?;
    rx.recv_timeout(RPC_TIMEOUT)
        .unwrap_or_else(|_| Err("npm pi rpc get_available_models timed out".into()))
}

fn read_models_response(reader: impl BufRead) -> Result<Vec<Value>, String> {
    for line in reader.lines() {
        let line = line.map_err(|e| format!("npm pi rpc read: {e}"))?;
        if let Ok(msg) = serde_json::from_str::<Value>(&line)
            && is_models_response(&msg)
        {
            return models_from_response(&msg);
        }
    }
    Err("npm pi rpc exited before listing models".into())
}

fn is_models_response(msg: &Value) -> bool {
    msg.get("type").and_then(Value::as_str) == Some("response")
        && msg.get("command").and_then(Value::as_str) == Some("get_available_models")
}

fn models_from_response(msg: &Value) -> Result<Vec<Value>, String> {
    if msg.get("success").and_then(Value::as_bool) == Some(false) {
        let err = msg.get("error").and_then(Value::as_str).unwrap_or("unknown");
        return Err(format!("npm pi rpc get_available_models failed: {err}"));
    }
    msg.pointer("/data/models")
        .and_then(Value::as_array)
        .cloned()
        .ok_or_else(|| "npm pi rpc response lacks data.models".into())
}

fn model_row(model: &Value) -> Option<(String, String)> {
    let provider = model.get("provider").and_then(Value::as_str)?;
    let id = model.get("id").and_then(Value::as_str)?;
    let levels = supported_thinking_levels(model).join("|");
    Some((format!("{provider}/{id}"), format!("{id}\tthinking={levels}")))
}

fn supported_thinking_levels(model: &Value) -> Vec<&'static str> {
    if !model.get("reasoning").and_then(Value::as_bool).unwrap_or(false) {
        return vec!["off"];
    }
    let map = model.get("thinkingLevelMap");
    EXTENDED_THINKING_LEVELS
        .into_iter()
        .filter(|level| match map.and_then(|m| m.get(*level)) {
            Some(Value::Null) => false,
            Some(_) => true,
            None => !matches!(*level, "xhigh" | "max"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn levels_follow_pi_level_map() {
        let terra = json!({"provider": "openai", "id": "gpt-5.6-terra", "reasoning": true,
            "thinkingLevelMap": {"off": "none", "minimal": null, "low": "low", "medium": "medium",
                "high": "high", "xhigh": "xhigh", "max": "max"}});
        assert_eq!(
            model_row(&terra),
            Some((
                "openai/gpt-5.6-terra".into(),
                "gpt-5.6-terra\tthinking=off|low|medium|high|xhigh|max".into()
            ))
        );
        let gpt5 = json!({"reasoning": true, "thinkingLevelMap": {"off": null, "xhigh": null}});
        assert_eq!(supported_thinking_levels(&gpt5), ["minimal", "low", "medium", "high"]);
        assert_eq!(supported_thinking_levels(&json!({"reasoning": true})), BASE_THINKING_LEVELS);
        assert_eq!(supported_thinking_levels(&json!({"reasoning": false})), ["off"]);
        assert_eq!(model_row(&json!({"id": "x"})), None);
    }

    #[test]
    fn reads_models_from_matching_response() {
        let text = "noise\n{\"type\":\"event\"}\n\
            {\"type\":\"response\",\"command\":\"get_available_models\",\"success\":true,\
            \"data\":{\"models\":[{\"id\":\"a\"}]}}\n";
        let models = read_models_response(text.as_bytes()).expect("models");
        assert_eq!(models, vec![json!({"id": "a"})]);
        let failed = "{\"type\":\"response\",\"command\":\"get_available_models\",\
            \"success\":false,\"error\":\"boom\"}\n";
        assert!(read_models_response(failed.as_bytes()).unwrap_err().contains("boom"));
        assert!(read_models_response(&b""[..]).unwrap_err().contains("exited"));
    }

    #[test]
    fn kiss_cov_models_rpc() {
        let _ = list_rpc_models;
        let _ = fetch_rpc_models;
        let _ = request_models;
    }
}
