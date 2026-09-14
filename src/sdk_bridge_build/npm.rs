use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use super::Bridge;

pub(super) fn resolve_npm() -> PathBuf {
    if let Some(p) = env::var_os("npm").filter(|v| !v.is_empty()) {
        return PathBuf::from(p);
    }
    which("npm").unwrap_or_else(|| {
        panic!(
            "malvin requires Node.js/npm to install the Cursor SDK and Cursor SDK bridges.\n\
             Install Node >= 22.13 (includes npm), ensure `npm` is on PATH, then re-run \
             `cargo install malvin` (or `cargo build`).\n\
             To compile the Rust binary without SDK bridges (agent backends will not work), \
             set MALVIN_SKIP_SDK_BRIDGES=1."
        )
    })
}

pub(super) fn which(bin: &str) -> Option<PathBuf> {
    let Ok(path) = env::var("PATH") else {
        return None;
    };
    for dir in env::split_paths(&path) {
        let candidate = dir.join(bin);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

pub(super) fn check_node_version(bridge: &Bridge) {
    let Some(node) = which("node") else {
        panic!(
            "malvin requires Node.js >= {}.{} for {} (node not found on PATH)",
            bridge.min_node.0, bridge.min_node.1, bridge.label
        );
    };
    let output = Command::new(&node)
        .arg("-v")
        .output()
        .unwrap_or_else(|e| panic!("failed to run `{} -v`: {e}", node.display()));
    assert!(output.status.success(), "`{} -v` failed", node.display());
    let text = String::from_utf8_lossy(&output.stdout);
    let (major, minor) = parse_node_version(text.trim()).unwrap_or_else(|| {
        panic!("could not parse Node version from {:?}", text.trim());
    });
    assert!(
        (major, minor) >= bridge.min_node,
        "malvin requires Node.js >= {}.{} for {} (found {}.{})",
        bridge.min_node.0,
        bridge.min_node.1,
        bridge.label,
        major,
        minor
    );
}

pub(super) fn parse_node_version(v: &str) -> Option<(u32, u32)> {
    let s = v.strip_prefix('v').unwrap_or(v);
    let mut parts = s.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    Some((major, minor))
}

pub(super) fn run_npm(npm: &Path, dir: &Path, args: &[&str]) {
    let status = npm_command(npm, dir, args)
        .status()
        .unwrap_or_else(|e| {
            panic!(
                "failed to run `{} {}` in {}: {e}",
                npm.display(),
                args.join(" "),
                dir.display()
            )
        });
    assert!(
        status.success(),
        "`{} {}` failed in {} (status {status}). \
         The {} npm dependency must install successfully for malvin agent backends.",
        npm.display(),
        args.join(" "),
        dir.display(),
        dir.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("sdk-bridge")
    );
}

fn npm_command(npm: &Path, dir: &Path, args: &[&str]) -> Command {
    let mut cmd = Command::new(npm);
    cmd.args(args).current_dir(dir).stdin(Stdio::null());
    attach_npm_stdio(&mut cmd);
    cmd
}

fn attach_npm_stdio(cmd: &mut Command) {
    #[cfg(unix)]
    {
        use std::os::fd::AsFd;
        if let Ok(out) = std::io::stderr().as_fd().try_clone_to_owned() {
            cmd.stdout(Stdio::from(out)).stderr(Stdio::inherit());
            return;
        }
    }
    cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
}
