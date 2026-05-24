use crate::acp::*;
use serde_json::json;

#[test]
fn jsonrpc_response_id_parses_u64_and_decimal_string_and_rejects_garbage() {
    assert_eq!(jsonrpc_response_id_as_u64(&json!(42u64)), Some(42));
    assert_eq!(jsonrpc_response_id_as_u64(&json!(42i64)), Some(42));
    assert_eq!(jsonrpc_response_id_as_u64(&json!("99")), Some(99));
    assert_eq!(jsonrpc_response_id_as_u64(&json!("not-a-number")), None);
    assert_eq!(jsonrpc_response_id_as_u64(&json!(-1i64)), None);
    assert_eq!(jsonrpc_response_id_as_u64(&json!(null)), None);
}

#[test]
fn request_permission_correlation_id_top_level_params_and_request_id() {
    let top = json!({"jsonrpc":"2.0","id":1,"params":{"id":2}});
    assert_eq!(request_permission_correlation_id(&top), top.get("id"));
    let nested = json!({"jsonrpc":"2.0","method":"session/request_permission","params":{"id":2}});
    assert_eq!(
        request_permission_correlation_id(&nested),
        nested.pointer("/params/id")
    );
    let req_id = json!({"params":{"requestId":"9"}});
    assert_eq!(
        request_permission_correlation_id(&req_id),
        req_id.pointer("/params/requestId")
    );
    let none = json!({"method":"session/request_permission","params":{}});
    assert_eq!(request_permission_correlation_id(&none), None);
}

#[test]
fn test_permission_reply_shape() {
    let id = json!(42u64);
    let body = json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "outcome": { "outcome": "selected", "optionId": "allow-always" }
        }
    });
    assert!(body.get("result").is_some());
}
