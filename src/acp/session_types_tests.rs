#[test]
fn prompt_trace_writer_and_acp_session_inner_type_names() {
    let _ = std::any::type_name::<crate::acp::PromptTraceWriter>();
    let _ = std::any::type_name::<crate::acp::session_types::AcpSessionInner>();
}

#[tokio::test]
async fn response_tx_oneshot_channel_constructible() {
    let (_tx, _rx): (crate::acp::session_types::ResponseTx, _) = tokio::sync::oneshot::channel();
}
