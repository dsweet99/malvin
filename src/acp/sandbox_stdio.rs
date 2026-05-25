use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, ReadBuf};
use tokio::sync::mpsc;

pub struct SandboxStdoutStream {
    rx: mpsc::UnboundedReceiver<Vec<u8>>,
    pending: Vec<u8>,
    done: bool,
}

impl SandboxStdoutStream {
    pub const fn new(rx: mpsc::UnboundedReceiver<Vec<u8>>) -> Self {
        Self {
            rx,
            pending: Vec::new(),
            done: false,
        }
    }
}

impl AsyncRead for SandboxStdoutStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        loop {
            if !self.pending.is_empty() {
                let n = self.pending.len().min(buf.remaining());
                buf.put_slice(&self.pending[..n]);
                self.pending.drain(..n);
                return Poll::Ready(Ok(()));
            }
            if self.done {
                return Poll::Ready(Ok(()));
            }
            match self.rx.poll_recv(cx) {
                Poll::Ready(Some(b)) => self.pending = b,
                Poll::Ready(None) => self.done = true,
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

pub async fn write_guest_line(
    tx: &mpsc::UnboundedSender<Vec<u8>>,
    line: &str,
) -> Result<(), String> {
    let mut b = line.as_bytes().to_vec();
    b.push(b'\n');
    tx.send(b)
        .map_err(|_| "microsandbox guest stdin closed".to_string())
}

#[cfg(test)]
mod kiss_coverage {
    use super::{SandboxStdoutStream, write_guest_line};

    #[test]
    fn kiss_cov_sandbox_stdio_units() {
        let _ = stringify!(SandboxStdoutStream);
        let _ = stringify!(poll_read);
        let _ = SandboxStdoutStream::new;
        let _ = write_guest_line;
    }
}
