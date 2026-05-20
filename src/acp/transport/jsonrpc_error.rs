// Pretty-print JSON-RPC error objects for logs.

pub(crate) fn format_jsonrpc_error(err: &Value) -> String {
    err.as_object()
        .map_or_else(|| err.to_string(), format_jsonrpc_error_obj)
}

pub(crate) fn format_jsonrpc_error_obj(obj: &Map<String, Value>) -> String {
    let code = jsonrpc_error_code_str(obj);
    let msg = jsonrpc_error_message_str(obj);
    let data_detail = jsonrpc_error_data_detail(obj);
    let mut parts = vec![format!("code={code}"), format!("message={msg:?}")];
    if let Some(d) = data_detail {
        parts.push(format!("detail={d:?}"));
    }
    parts.join("; ")
}

pub(crate) fn jsonrpc_error_code_str(obj: &Map<String, Value>) -> String {
    obj.get("code")
        .map_or_else(|| "null".to_string(), ToString::to_string)
}

pub(crate) fn jsonrpc_error_message_str(obj: &Map<String, Value>) -> &str {
    obj.get("message")
        .and_then(|m| m.as_str())
        .unwrap_or("")
}

pub(crate) fn jsonrpc_error_data_detail(obj: &Map<String, Value>) -> Option<&str> {
    obj.get("data")
        .and_then(|d| d.get("message"))
        .and_then(|m| m.as_str())
        .or_else(|| obj.get("data").and_then(Value::as_str))
}

#[test]
fn format_jsonrpc_error_includes_code_and_message() {
    use serde_json::json;
    let _ = format_jsonrpc_error_obj;
    let _ = jsonrpc_error_code_str;
    let _ = jsonrpc_error_message_str;
    let _ = jsonrpc_error_data_detail;
    let err = json!({"code": -32600, "message": "invalid"});
    let formatted = format_jsonrpc_error(&err);
    assert!(formatted.contains("code=-32600"));
    assert!(formatted.contains("invalid"));
}
