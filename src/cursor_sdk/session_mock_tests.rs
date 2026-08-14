
use std::path::PathBuf;
use std::process::Stdio;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};

fn mock_bridge_js() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/cursor_sdk/mock_bridge.js")
}

async fn spawn_mock() -> (Child, tokio::process::ChildStdin, BufReader<tokio::process::ChildStdout>) {
    let node = super::node_resolve::resolve_node_bin().expect("modern node for mock bridge");
    let mut child = Command::new(node)
        .arg(mock_bridge_js())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn mock");
    let stdin = child.stdin.take().expect("stdin");
    let stdout = BufReader::new(child.stdout.take().expect("stdout"));
    (child, stdin, stdout)
}

async fn cursor_mock_write_line(stdin: &mut tokio::process::ChildStdin, line: &str) {
    stdin.write_all(line.as_bytes()).await.unwrap();
    stdin.flush().await.unwrap();
}

async fn read_until(reader: &mut BufReader<tokio::process::ChildStdout>, needle: &str) -> String {
    let mut line = String::new();
    for _ in 0..8 {
        line.clear();
        let n = reader.read_line(&mut line).await.unwrap();
        if n == 0 || line.contains(needle) {
            break;
        }
    }
    line
}

#[tokio::test]
async fn mock_bridge_create_send_close() {
    assert!(mock_bridge_js().is_file(), "mock_bridge.js missing");
    let (mut child, mut stdin, mut reader) = spawn_mock().await;
    cursor_mock_write_line(
        &mut stdin,
        "{\"op\":\"create\",\"cwd\":\"/tmp\",\"model\":\"auto\",\"apiKey\":\"k\"}\n",
    )
    .await;
    let ok = read_until(&mut reader, "\"ok\"").await;
    assert!(ok.contains("\"event\":\"ok\""), "{ok}");
    cursor_mock_write_line(&mut stdin, "{\"op\":\"send\",\"prompt\":\"hello\"}\n").await;
    let done = read_until(&mut reader, "run_done").await;
    assert!(done.contains("inputTokens"), "{done}");
    cursor_mock_write_line(&mut stdin, "{\"op\":\"close\"}\n").await;
    let _ = child.wait().await;
}
