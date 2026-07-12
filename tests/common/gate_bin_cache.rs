//! Process-wide cached fake gate-tool bins for integration subprocess tests.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use super::workspace::{chmod755, mock_cache_root, write_fake_kiss, write_failing_command};

#[cfg(unix)]
pub fn static_fake_kiss_path_var() -> String {
    static PATH: OnceLock<String> = OnceLock::new();
    PATH.get_or_init(|| {
        let dir = static_gate_bin_subdir("pass");
        let kiss = dir.join("kiss");
        if !kiss.is_file() {
            write_fake_kiss(&kiss);
        }
        minimal_gate_path_var(&dir)
    })
    .clone()
}

#[cfg(unix)]
pub fn static_failing_gates_path_var() -> String {
    static PATH: OnceLock<String> = OnceLock::new();
    PATH.get_or_init(|| {
        let dir = static_gate_bin_subdir("fail");
        for name in ["kiss", "lint", "gate_b"] {
            let path = dir.join(name);
            if !path.is_file() {
                write_failing_command_env_trace(&path, name);
            }
        }
        minimal_gate_path_var(&dir)
    })
    .clone()
}

#[cfg(unix)]
pub fn write_failing_gate_tools(bin_dir: &Path, trace: &Path) {
    for name in ["kiss", "lint", "gate_b"] {
        write_failing_command(&bin_dir.join(name), trace);
    }
}

#[cfg(unix)]
fn write_failing_command_env_trace(path: &Path, tool_name: &str) {
    // Keep shell `${VAR:-}` out of the `format!` template so clippy does not treat it as a Rust format arg.
    const TRACE_ASSIGN: &str = r#"trace="${MALVIN_TEST_GATE_TRACE:-}""#;
    std::fs::write(
        path,
        format!(
            "#!/usr/bin/env sh\n\
{TRACE_ASSIGN}\n\
if [ -n \"$trace\" ]; then echo \"{tool_name} $@\" >> \"$trace\"; fi\n\
exit 1\n"
        ),
    )
    .expect("write failing command");
    chmod755(path);
}

#[cfg(unix)]
fn minimal_gate_path_var(bin_dir: &Path) -> String {
    format!("{}:/usr/bin:/bin", bin_dir.display())
}

#[cfg(unix)]
fn static_gate_bin_subdir(name: &str) -> PathBuf {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    let root = ROOT.get_or_init(|| mock_cache_root().path().join("static-gate-bin"));
    let dir = root.join(name);
    std::fs::create_dir_all(&dir).expect("mkdir static gate bin subdir");
    dir
}
