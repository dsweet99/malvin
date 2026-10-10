use std::io::{BufRead, BufReader, Lines, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{Value, json};

use super::remote_output::{RemoteLines, Stream};
use crate::npm_bridge_install::NpmBridgePackage;

macro_rules! payload_file {
    ($rel:literal) => {
        (
            $rel,
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/modal-bridge/", $rel)).as_slice(),
        )
    };
}

pub(crate) const PACKAGE: NpmBridgePackage = NpmBridgePackage {
    dir_name: "modal-bridge",
    sdk_marker: "node_modules/modal/package.json",
    npm_label: "Modal JS SDK (modal)",
    payload: &[
        payload_file!("package.json"),
        payload_file!("package-lock.json"),
        payload_file!("dist/bridge.js"),
        payload_file!("dist/dispatch.js"),
        payload_file!("dist/modal_like.js"),
        payload_file!("dist/ops.js"),
        payload_file!("dist/protocol.js"),
        payload_file!("dist/toolchain.js"),
    ],
};

pub fn ensure_installed() -> Result<PathBuf, String> {
    PACKAGE
        .install_into(&PACKAGE.default_install_dir())
        .map_err(|e| {
            format!(
                "malvin --remote=modal:sandbox needs Node.js >= 22.13 with npm to install the Modal bridge: {e}"
            )
        })
}

pub struct ModalBridge {
    child: Child,
    stdin: Option<ChildStdin>,
    lines: Lines<BufReader<ChildStdout>>,
    next_id: u64,
    remote: RemoteLines,
}

pub fn node_bridge_command(bridge_js: &Path) -> Result<Command, String> {
    let node = crate::backends::cursor_sdk::node_resolve::resolve_node_bin()?;
    let mut cmd = Command::new(node);
    crate::backends::cursor_sdk::node_resolve::apply_quiet_node_cli_std(&mut cmd);
    cmd.arg(bridge_js);
    Ok(cmd)
}

fn relay_event(remote: &mut RemoteLines, event: &str, data: &str) {
    match event {
        "stdout" => remote.push(Stream::Stdout, data),
        "stderr" => remote.push(Stream::Stderr, data),
        _ => super::remote_output::print_status(data),
    }
}

impl ModalBridge {
    pub fn spawn(mut cmd: Command) -> Result<Self, String> {
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        let mut child = cmd
            .spawn()
            .map_err(|e| format!("failed to start the Modal bridge: {e}"))?;
        let stdin = child.stdin.take();
        let stdout = child.stdout.take().ok_or("Modal bridge has no stdout")?;
        Ok(Self {
            child,
            stdin,
            lines: BufReader::new(stdout).lines(),
            next_id: 1,
            remote: RemoteLines::default(),
        })
    }

    pub fn exited(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(Some(_)))
    }

    fn send(&mut self, id: u64, op: &str, mut fields: Value) -> Result<(), String> {
        fields["id"] = json!(id);
        fields["op"] = json!(op);
        let stdin = self.stdin.as_mut().ok_or("Modal bridge stdin is closed")?;
        writeln!(stdin, "{fields}")
            .and_then(|()| stdin.flush())
            .map_err(|e| format!("write to the Modal bridge: {e}"))
    }

    pub fn call(&mut self, op: &str, fields: Value) -> Result<Value, String> {
        let id = self.next_id;
        self.next_id += 1;
        self.send(id, op, fields)?;
        let reply = self.read_reply(op);
        self.remote.flush();
        reply
    }

    fn read_reply(&mut self, op: &str) -> Result<Value, String> {
        loop {
            let line = self
                .lines
                .next()
                .ok_or_else(|| format!("the Modal bridge exited during `{op}`"))?
                .map_err(|e| format!("read from the Modal bridge: {e}"))?;
            let msg: Value = serde_json::from_str(&line)
                .map_err(|e| format!("bad line from the Modal bridge ({e}): {line}"))?;
            if let Some(event) = msg.get("event").and_then(Value::as_str) {
                relay_event(
                    &mut self.remote,
                    event,
                    msg["data"].as_str().unwrap_or_default(),
                );
                continue;
            }
            return reply_result(op, msg);
        }
    }
}

fn reply_result(op: &str, msg: Value) -> Result<Value, String> {
    if msg["ok"].as_bool() == Some(true) {
        return Ok(msg);
    }
    let error = msg["error"].as_str().unwrap_or("unknown error");
    Err(format!("Modal `{op}` failed: {error}"))
}

impl Drop for ModalBridge {
    fn drop(&mut self) {
        drop(self.stdin.take());
        let _ = self.child.wait();
    }
}
