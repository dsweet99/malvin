#[test]
fn prompt_trace_writer_and_acp_session_inner_type_names() {
    let _ = std::any::type_name::<crate::acp::PromptTraceWriter>();
    let _ = std::any::type_name::<crate::acp::session_types::AcpSessionInner>();
}

#[tokio::test]
async fn response_tx_oneshot_channel_constructible() {
    let (tx, rx): (crate::acp::ResponseTx, _) = tokio::sync::oneshot::channel();
    tx.send(Ok(serde_json::json!({}))).expect("send");
    let _ = rx.await.expect("recv");
}
