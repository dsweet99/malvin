pub(crate) const fn prompt_stdout_replacement(_who: &str) -> Option<&'static str> {
    None
}

pub(crate) async fn rpc_session_prompt_text(
    session: &AcpSession,
    text: &str,
    id: u64,
) -> Result<(), String> {
    let params = serde_json::json!({
        "sessionId": &session.0.session_id,
        "prompt": [{ "type": "text", "text": text }]
    });
    let io = acp_stdio(&session.0);
    let child_pid = session
        .0
        .child
        .lock()
        .await
        .as_ref()
        .and_then(tokio::process::Child::id);
    rpc_request_with_correlation_id(RpcOutgoing {
        io: &io,
        id,
        method: "session/prompt",
        params,
        rpc_timeout: session.0.rpc_timeout,
        child_pid,
    })
    .await
    .map(|_| ())
}

pub(crate) async fn do_split_trace_preamble(
    file: &mut tokio::fs::File,
    _raw_output: bool,
    split: &DoPromptTraceSplit<'_>,
) -> Result<(), String> {
    trace_write_invocation_and_do_split_prompt(file, split).await?;
    Ok(())
}
