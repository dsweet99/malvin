#[cfg(unix)]
mod incoming_line_parse_error {
    use crate::acp::handle_incoming_line;
    use crate::acp::ResponseTx;
    use super::super::acp_activity_state;
    use crate::acp_test_unix_bin::unix_bin_with_fallback;
    use std::collections::HashMap;
    use std::process::Stdio;
    use std::sync::Arc;
    use std::sync::atomic::Ordering;
    use tokio::process::Command;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_handle_incoming_line_parse_error_and_extension_method() {
        let pending: Arc<Mutex<HashMap<u64, ResponseTx>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let (acp_activity_seq, acp_activity_notify) = acp_activity_state();
        let mut child = Command::new(unix_bin_with_fallback("sleep"))
            .arg("8")
            .stdin(Stdio::piped())
            .spawn()
            .expect("sleep");
        let stdin = Arc::new(Mutex::new(child.stdin.take().expect("stdin")));
        let _reap = tokio::spawn(async move {
            let _ = child.kill().await;
            let _ = child.wait().await;
        });

        handle_incoming_line(
            "%%%",
            crate::acp::IncomingLineDispatch {
                pending: &pending,
                stdin: &stdin,
                acp_activity_seq: &acp_activity_seq,
                acp_activity_notify: &acp_activity_notify,
                prompt_cleanup: None,
                acp_verbose: false,
                trace_jsonl: None,
            },
        )
        .await;
        handle_incoming_line(
            r#"{"jsonrpc":"2.0","method":"cursor/task","params":{}}"#,
            crate::acp::IncomingLineDispatch {
                pending: &pending,
                stdin: &stdin,
                acp_activity_seq: &acp_activity_seq,
                acp_activity_notify: &acp_activity_notify,
                prompt_cleanup: None,
                acp_verbose: false,
                trace_jsonl: None,
            },
        )
        .await;
        assert_eq!(
            acp_activity_seq.load(Ordering::SeqCst),
            1,
            "only valid JSON should count as ACP activity"
        );
    }
}
