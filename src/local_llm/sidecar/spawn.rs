//! Spawn helpers for the local MLX sidecar process.

use std::fs;
use std::fs::File;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use super::super::download::{local_llm_script, resolve_python};
use super::super::registry::LocalModelSpec;
use crate::workspace_paths::malvin_user_home_root;

pub(super) fn free_loopback_port() -> Result<u16, String> {
    let listener =
        TcpListener::bind("127.0.0.1:0").map_err(|e| format!("bind loopback port: {e}"))?;
    let port = listener
        .local_addr()
        .map_err(|e| format!("local_addr: {e}"))?
        .port();
    drop(listener);
    Ok(port)
}

pub(super) fn sidecar_run_dir(slug: &str, port: u16) -> Result<PathBuf, String> {
    let dir = malvin_user_home_root()
        .join("local_sidecar")
        .join(format!("{slug}-{port}"));
    fs::create_dir_all(&dir).map_err(|e| format!("sidecar run dir {}: {e}", dir.display()))?;
    Ok(dir)
}

pub(super) fn spawn_sidecar_process(
    spec: &LocalModelSpec,
    model_dir: &Path,
    port: u16,
    run_dir: &Path,
) -> Result<Child, String> {
    let (stdout_file, stderr_file) = open_sidecar_logs(run_dir)?;
    let script = local_llm_script("server.py")?;
    let python = resolve_python()?;
    launch_sidecar(LaunchArgs {
        python: &python,
        script: &script,
        model_dir,
        port,
        spec,
        stdout_file,
        stderr_file,
    })
}

struct LaunchArgs<'a> {
    python: &'a Path,
    script: &'a Path,
    model_dir: &'a Path,
    port: u16,
    spec: &'a LocalModelSpec,
    stdout_file: File,
    stderr_file: File,
}

fn open_sidecar_logs(run_dir: &Path) -> Result<(File, File), String> {
    let stdout_path = run_dir.join("sidecar.stdout");
    let stderr_path = run_dir.join("sidecar.stderr");
    let stdout_file = File::create(&stdout_path)
        .map_err(|e| format!("sidecar stdout {}: {e}", stdout_path.display()))?;
    let stderr_file = File::create(&stderr_path)
        .map_err(|e| format!("sidecar stderr {}: {e}", stderr_path.display()))?;
    Ok((stdout_file, stderr_file))
}

fn launch_sidecar(args: LaunchArgs<'_>) -> Result<Child, String> {
    // Intentionally not malvin_std_command: sidecar must stay outside sandbox RSS.
    Command::new(args.python)
        .arg(args.script)
        .arg("--host")
        .arg("127.0.0.1")
        .arg("--port")
        .arg(args.port.to_string())
        .arg("--model-dir")
        .arg(args.model_dir)
        .arg("--model-id")
        .arg(args.spec.slug)
        .arg("--loader")
        .arg(args.spec.loader)
        .stdin(Stdio::null())
        .stdout(args.stdout_file)
        .stderr(args.stderr_file)
        .spawn()
        .map_err(|e| format!("failed to spawn local sidecar: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_loopback_port_returns_nonzero() {
        assert!(free_loopback_port().expect("port") > 0);
    }

    #[test]
    fn sidecar_run_dir_creates_under_malvin_home() {
        let dir = sidecar_run_dir("unit_test_slug", 9).expect("dir");
        assert!(dir.ends_with("unit_test_slug-9"));
        assert!(dir.is_dir());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn kiss_witness_launch_args_type() {
        let tmp = tempfile::tempdir().expect("tmp");
        let stdout = File::create(tmp.path().join("o")).expect("o");
        let stderr = File::create(tmp.path().join("e")).expect("e");
        let spec = crate::local_llm::lookup_local_model("qwen35_9b_q4").expect("spec");
        let args = LaunchArgs {
            python: Path::new("python3"),
            script: Path::new("server.py"),
            model_dir: tmp.path(),
            port: 9,
            spec,
            stdout_file: stdout,
            stderr_file: stderr,
        };
        assert_eq!(args.port, 9);
        assert_eq!(args.spec.slug, "qwen35_9b_q4");
    }
}
