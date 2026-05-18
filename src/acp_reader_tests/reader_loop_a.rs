use serde_json::json;

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

#[cfg(unix)]
mod incoming_line_unix {
    use crate::acp::handle_incoming_line;
    use crate::acp::ResponseTx;
    use super::super::{
        CAT_BIN, UnixCatIncoming, acp_activity_state, incoming_permission_dispatch_plain,
        unix_cat_stdio_incoming, unix_true_exited_stdio_stdin_only,
    };
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::sync::atomic::Ordering;
    use tokio::io::AsyncReadExt;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_handle_session_update_and_permission_replies() {
        let UnixCatIncoming {
            mut child,
            stdin,
            mut stdout,
            pending,
            acp_activity_seq,
            acp_activity_notify,
        } = unix_cat_stdio_incoming(CAT_BIN).await;

        handle_incoming_line(
            r#"{"jsonrpc":"2.0","method":"session/update","params":{"t":1}}"#,
            incoming_permission_dispatch_plain(
                &pending,
                &stdin,
                &acp_activity_seq,
                &acp_activity_notify,
            ),
        )
        .await;

        handle_incoming_line(
            r#"{"jsonrpc":"2.0","id":42,"method":"session/request_permission","params":{}}"#,
            incoming_permission_dispatch_plain(
                &pending,
                &stdin,
                &acp_activity_seq,
                &acp_activity_notify,
            ),
        )
        .await;

        drop(stdin);
        let mut received = Vec::new();
        stdout
            .read_to_end(&mut received)
            .await
            .expect("read stdout");
        let _ = child.wait().await.expect("wait cat");
        let line = String::from_utf8_lossy(&received);
        assert!(
            line.contains("allow-always")
                && (line.contains(r#""id":42"#) || line.contains(r#""id": 42"#)),
            "expected allow-always reply echoing id 42; got {line:?}"
        );
        assert_eq!(
            acp_activity_seq.load(Ordering::SeqCst),
            2,
            "both JSON messages should count as ACP activity"
        );
    }

    /// KPOP: `session/request_permission` with no correlation id anywhere still skips `write_rpc_line`.
    #[tokio::test]
    async fn kpop_permission_without_correlation_id_writes_nothing_to_child_stdin() {
        let UnixCatIncoming {
            mut child,
            stdin,
            mut stdout,
            pending,
            acp_activity_seq,
            acp_activity_notify,
        } = unix_cat_stdio_incoming(CAT_BIN).await;

        handle_incoming_line(
            r#"{"jsonrpc":"2.0","method":"session/request_permission","params":{}}"#,
            incoming_permission_dispatch_plain(
                &pending,
                &stdin,
                &acp_activity_seq,
                &acp_activity_notify,
            ),
        )
        .await;

        drop(stdin);
        let mut received = Vec::new();
        stdout
            .read_to_end(&mut received)
            .await
            .expect("read stdout");
        let _ = child.wait().await.expect("wait cat");
        assert!(
            received.is_empty(),
            "expected no bytes written for permission message without id; got {:?}",
            String::from_utf8_lossy(&received)
        );
    }

    /// Permission prompt with `id` only under `params` must still get an allow-always JSON-RPC reply line.
    #[tokio::test]
    async fn permission_with_id_in_params_writes_allow_always_reply_line() {
        let UnixCatIncoming {
            mut child,
            stdin,
            mut stdout,
            pending,
            acp_activity_seq,
            acp_activity_notify,
        } = unix_cat_stdio_incoming(CAT_BIN).await;

        handle_incoming_line(
            r#"{"jsonrpc":"2.0","method":"session/request_permission","params":{"id":77}}"#,
            incoming_permission_dispatch_plain(
                &pending,
                &stdin,
                &acp_activity_seq,
                &acp_activity_notify,
            ),
        )
        .await;

        drop(stdin);
        let mut received = Vec::new();
        stdout
            .read_to_end(&mut received)
            .await
            .expect("read stdout");
        let _ = child.wait().await.expect("wait cat");
        let line = String::from_utf8_lossy(&received);
        assert!(
            line.contains("allow-always")
                && (line.contains(r#""id":77"#) || line.contains(r#""id": 77"#)),
            "expected allow-always reply echoing id 77; got {line:?}"
        );
    }

    #[tokio::test]
    async fn test_permission_json_or_write_failure_is_logged() {
        let pending: Arc<Mutex<HashMap<u64, ResponseTx>>> = Arc::new(Mutex::new(HashMap::new()));
        let (acp_activity_seq, acp_activity_notify) = acp_activity_state();
        let stdin = unix_true_exited_stdio_stdin_only().await;
        handle_incoming_line(
            r#"{"jsonrpc":"2.0","id":9,"method":"session/request_permission","params":{}}"#,
            incoming_permission_dispatch_plain(
                &pending,
                &stdin,
                &acp_activity_seq,
                &acp_activity_notify,
            ),
        )
        .await;
        assert_eq!(
            acp_activity_seq.load(Ordering::SeqCst),
            1,
            "permission request should count as ACP activity even when reply write fails"
        );
        assert!(
            pending.lock().await.is_empty(),
            "permission write failure must not leak pending RPC state"
        );
    }
}

#[cfg(unix)]
pub(super) use incoming_line_unix::*;
