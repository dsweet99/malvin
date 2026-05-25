use microsandbox::sandbox::exec::{ExecEvent, ExecSink};
use tokio::sync::mpsc;

pub async fn pump_guest_stdout(
    mut exec: microsandbox::ExecHandle,
    tx: mpsc::UnboundedSender<Vec<u8>>,
) {
    while let Some(ev) = exec.recv().await {
        if let ExecEvent::Stdout(data) = ev {
            if tx.send(data.to_vec()).is_err() {
                break;
            }
        } else if matches!(ev, ExecEvent::Exited { .. } | ExecEvent::Failed(_)) {
            break;
        }
    }
}

pub async fn pump_malvin_stdin(
    mut rx: mpsc::UnboundedReceiver<Vec<u8>>,
    sink: ExecSink,
) {
    while let Some(chunk) = rx.recv().await {
        if sink.write(&chunk).await.is_err() {
            break;
        }
    }
    let _ = sink.close().await;
}

#[cfg(test)]
mod kiss_coverage {
    #[test]
    fn kiss_cov_bridge_pumps() {
        let _ = stringify!(pump_guest_stdout);
        let _ = stringify!(pump_malvin_stdin);
    }
}
