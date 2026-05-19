#[cfg(unix)]
mod unix {
    use crate::acp::reader_tests_helpers::{CatSession, IncomingDispatchParts, acp_activity_state};
    use std::sync::atomic::Ordering;

    #[tokio::test]
    async fn test_handle_session_update_and_permission_replies() {
        let cat = CatSession::new().await;
        cat.dispatch_parts()
            .dispatch_lines(&[
                r#"{"jsonrpc":"2.0","method":"session/update","params":{"t":1}}"#,
                r#"{"jsonrpc":"2.0","id":42,"method":"session/request_permission","params":{}}"#,
            ])
            .await;
        assert_eq!(
            cat.acp_activity_seq.load(Ordering::SeqCst),
            2,
            "both JSON messages should count as ACP activity"
        );
        let line = cat.finish_stdout().await;
        assert!(
            line.contains("allow-always")
                && (line.contains(r#""id":42"#) || line.contains(r#""id": 42"#)),
            "expected allow-always reply echoing id 42; got {line:?}"
        );
    }

    #[tokio::test]
    async fn kpop_permission_without_correlation_id_writes_nothing_to_child_stdin() {
        let cat = CatSession::new().await;
        cat.dispatch_parts()
            .dispatch_lines(&[r#"{"jsonrpc":"2.0","method":"session/request_permission","params":{}}"#])
            .await;
        let line = cat.finish_stdout().await;
        assert!(line.is_empty(), "expected no bytes written for permission without id; got {line:?}");
    }

    #[tokio::test]
    async fn permission_with_id_in_params_writes_allow_always_reply_line() {
        let cat = CatSession::new().await;
        cat.dispatch_parts()
            .dispatch_lines(&[r#"{"jsonrpc":"2.0","method":"session/request_permission","params":{"id":77}}"#])
            .await;
        let line = cat.finish_stdout().await;
        assert!(
            line.contains("allow-always")
                && (line.contains(r#""id":77"#) || line.contains(r#""id": 77"#)),
            "expected allow-always reply echoing id 77; got {line:?}"
        );
    }

    #[tokio::test]
    async fn test_permission_json_or_write_failure_is_logged() {
        let pending = std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));
        let (acp_activity_seq, acp_activity_notify) = acp_activity_state();
        let mut child = tokio::process::Command::new(crate::acp_test_unix_bin::unix_bin_with_fallback("true"))
            .stdin(std::process::Stdio::piped())
            .spawn()
            .expect("true");
        let stdin = std::sync::Arc::new(tokio::sync::Mutex::new(child.stdin.take().expect("stdin")));
        let _ = child.wait().await;
        IncomingDispatchParts {
            pending: &pending,
            stdin: &stdin,
            acp_activity_seq: &acp_activity_seq,
            acp_activity_notify: &acp_activity_notify,
        }
        .dispatch_lines(&[r#"{"jsonrpc":"2.0","id":9,"method":"session/request_permission","params":{}}"#])
        .await;
        assert_eq!(acp_activity_seq.load(Ordering::SeqCst), 1);
        assert!(pending.lock().await.is_empty());
    }
}
