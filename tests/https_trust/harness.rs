//! Issue 45: a private CA trusted through the system-trust configuration must
//! complete a TLS handshake, and the default `WebPKI` configuration must reject it.
//!
//! Each integration binary calls one entry point, so the other entry point and
//! some server slots are unused in that binary.

#![allow(dead_code)]

mod certs;

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

const SERVER_SCRIPT: &str = r#"
import http.server
import ssl
import sys

context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
context.load_cert_chain(sys.argv[1], sys.argv[2])

class Handler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        body = b"ok"
        self.send_response(200)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, format, *args):
        return

server = http.server.HTTPServer(("127.0.0.1", 0), Handler)
server.socket = context.wrap_socket(server.socket, server_side=True)
print(server.server_address[1], flush=True)
server.serve_forever()
"#;

struct Material {
    servers: Servers,
    ca_pem: PathBuf,
    _root: tempfile::TempDir,
}

struct TlsServer {
    child: Child,
    port: u16,
}

impl Drop for TlsServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

struct Servers {
    good: TlsServer,
    wrong_host: TlsServer,
    expired: TlsServer,
    untrusted: TlsServer,
}

pub fn assert_system_trust_accepts_private_ca() {
    let fixture = prepare_fixture();
    clear_trust_env();
    set_env("REQUESTS_CA_BUNDLE", &fixture.ca_pem);
    let good = https_get(fixture.servers.good.port);
    assert!(
        good.is_ok(),
        "system trust should accept the private CA, got {good:?}"
    );
    assert_rejected(fixture.servers.wrong_host.port, "notvalidforname");
    assert_rejected(fixture.servers.expired.port, "expired");
    assert_rejected(fixture.servers.untrusted.port, "unknownissuer");
}

pub fn assert_webpki_rejects_private_ca() {
    let fixture = prepare_fixture();
    clear_trust_env();
    set_env("PI_HTTP_USE_SYSTEM_CERTS", "0");
    set_env("REQUESTS_CA_BUNDLE", &fixture.ca_pem);
    assert_rejected(fixture.servers.good.port, "unknownissuer");
}

fn prepare_fixture() -> Material {
    let root = tempfile::tempdir().expect("tempdir");
    let dir = root.path();
    certs::build_certs(dir);
    let script = dir.join("server.py");
    fs::write(&script, SERVER_SCRIPT).expect("write server script");
    let servers = start_servers(dir, &script);
    Material {
        servers,
        ca_pem: dir.join("ca.pem"),
        _root: root,
    }
}

fn start_servers(dir: &Path, script: &Path) -> Servers {
    let key = dir.join("server.key");
    Servers {
        good: spawn_server(&dir.join("good.pem"), &key, script),
        wrong_host: spawn_server(&dir.join("wrong.pem"), &key, script),
        expired: spawn_server(&dir.join("expired.pem"), &key, script),
        untrusted: spawn_server(&dir.join("untrusted.pem"), &key, script),
    }
}

fn spawn_server(cert: &Path, key: &Path, script: &Path) -> TlsServer {
    let mut child = Command::new("python3")
        .arg(script)
        .arg(cert)
        .arg(key)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("python3");
    let stdout = child.stdout.take().expect("server stdout");
    let mut line = String::new();
    let read = BufReader::new(stdout).read_line(&mut line);
    if read.unwrap_or(0) == 0 {
        let _ = child.kill();
        let output = child.wait_with_output().expect("server output");
        panic!(
            "tls server exited: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let port = line.trim().parse().expect("server port");
    TlsServer { child, port }
}

fn set_env(key: &str, value: impl AsRef<std::ffi::OsStr>) {
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var(key, value);
    }
}

fn clear_trust_env() {
    for key in [
        "SSL_CERT_FILE",
        "SSL_CERT_DIR",
        "REQUESTS_CA_BUNDLE",
        "CURL_CA_BUNDLE",
        "PIP_CERT",
        "PI_HTTP_USE_SYSTEM_CERTS",
    ] {
        #[allow(unsafe_code)]
        unsafe {
            std::env::remove_var(key);
        }
    }
}

fn https_get(port: u16) -> Result<u16, String> {
    let runtime = asupersync::runtime::RuntimeBuilder::current_thread()
        .build()
        .map_err(|err| err.to_string())?;
    let url = format!("https://127.0.0.1:{port}/");
    runtime.block_on(async move {
        pi::http::client::Client::new()
            .get(&url)
            .timeout(Duration::from_secs(2))
            .send()
            .await
            .map(|response| response.status())
            .map_err(|err| err.to_string())
    })
}

fn assert_rejected(port: u16, needle: &str) {
    let err = https_get(port).expect_err("handshake should fail");
    assert!(
        compact(&err).contains(needle),
        "error `{err}` should contain `{needle}`"
    );
}

fn compact(text: &str) -> String {
    text.chars()
        .filter(|ch| !ch.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect()
}
