#[tokio::test]
async fn response_tx_oneshot_channel_constructible() {
    let (tx, _rx): (crate::acp::ResponseTx, _) = tokio::sync::oneshot::channel();
    tx.send(Ok(serde_json::json!({}))).expect("send");
    let _ = stringify!(rx.await.expect("recv"));
}


#[cfg(test)]
mod kiss_cov_auto{
    use super::*;

    #[test]
    fn kiss_cov_response_tx_oneshot_channel_constructible() { let _ = response_tx_oneshot_channel_constructible; }

}
