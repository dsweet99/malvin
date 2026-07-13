//! Shell out to `malvin init` so checks discovery runs in a separate OS process.

use std::path::Path;
use std::process::Command;

use crate::malvin_sandbox::malvin_std_command;

use super::{checks_already_valid, finish_checks_discovery, SharedOpts};

fn malvin_self_executable() -> Result<std::path::PathBuf, String> {
    if let Ok(bin) = std::env::var("MALVIN_BIN") {
        if !bin.is_empty() {
            return Ok(std::path::PathBuf::from(bin));
        }
    }
    std::env::current_exe().map_err(|e| format!("malvin init subprocess: current_exe: {e}"))
}

pub(super) fn append_subprocess_shared_opts(cmd: &mut Command, shared: &SharedOpts) {
    cmd.args(["--model", &shared.model]);
    if shared.no_force {
        cmd.arg("--no-force");
    }
    if shared.no_tenacious {
        cmd.arg("--no-tenacious");
    }
    if shared.no_tee {
        cmd.arg("--no-tee");
    }
    if shared.no_markdown {
        cmd.arg("--no-markdown");
    }
    if shared.verbose {
        cmd.arg("--verbose");
    }
    cmd.args(["--max-acp-retries", &shared.max_acp_retries.to_string()]);
    if crate::model_id::uses_mini_backend(&shared.model) {
        cmd.args([
            "--mini-max-http-turns",
            &shared.mini_max_http_turns.to_string(),
        ]);
        cmd.args([
            "--mini-max-bash-execs",
            &shared.mini_max_bash_execs.to_string(),
        ]);
        cmd.args([
            "--mini-max-http-retries",
            &shared.mini_max_http_retries.to_string(),
        ]);
        cmd.args([
            "--mini-max-gate-retries",
            &shared.mini_max_gate_retries.to_string(),
        ]);
        cmd.args([
            "--mini-max-shrink-passes",
            &shared.mini_max_shrink_passes.to_string(),
        ]);
    }
    if let Some(name) = &shared.name {
        cmd.args(["--name", name]);
    }
}

pub(super) fn build_malvin_init_subprocess(
    work_dir: &Path,
    shared: &SharedOpts,
) -> Result<Command, String> {
    let exe = malvin_self_executable()?;
    let mut cmd = malvin_std_command(exe);
    cmd.current_dir(work_dir);
    append_subprocess_shared_opts(&mut cmd, shared);
    cmd.args(["--background", "init"]);
    Ok(cmd)
}

pub(super) fn run_checks_discovery_init_subprocess(
    work_dir: &Path,
    shared: &SharedOpts,
) -> Result<(), String> {
    if checks_already_valid(work_dir)? {
        return Ok(());
    }
    let output = build_malvin_init_subprocess(work_dir, shared)?
        .output()
        .map_err(|e| format!("malvin init subprocess: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let detail = stderr.trim();
        let msg = if detail.is_empty() {
            format!("malvin init subprocess exited with {}", output.status)
        } else {
            format!("malvin init subprocess exited with {}: {detail}", output.status)
        };
        return Err(msg);
    }
    finish_checks_discovery(work_dir)
}

pub(crate) async fn ensure_malvin_checks_discovered_via_init_subprocess(
    work_dir: &Path,
    shared: &SharedOpts,
) -> Result<(), String> {
    let work_dir = work_dir.to_path_buf();
    let shared = shared.clone();
    tokio::task::spawn_blocking(move || run_checks_discovery_init_subprocess(&work_dir, &shared))
        .await
        .map_err(|e| format!("malvin init subprocess task: {e}"))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_malvin_init_subprocess_includes_init_and_background() {
        crate::test_utils::with_isolated_home(|work| {
            let shared = SharedOpts::test_defaults();
            let cmd = build_malvin_init_subprocess(work, &shared).expect("build");
            let program = cmd.get_program().to_string_lossy();
            assert!(
                program.contains("malvin") || std::path::Path::new(program.as_ref()).exists(),
                "program: {program}"
            );
            assert_eq!(cmd.get_current_dir(), Some(work));
            let args: Vec<_> = cmd
                .get_args()
                .map(|a| a.to_string_lossy().into_owned())
                .collect();
            assert!(args.contains(&"--background".to_string()));
            assert!(args.contains(&"init".to_string()));
            assert!(args.contains(&"--no-force".to_string()));
        });
    }

    #[test]
    fn run_checks_discovery_init_subprocess_skips_when_checks_valid() {
        crate::test_utils::with_isolated_home(|work| {
            crate::seed_malvin_checks(work, "make lint\n");
            let shared = SharedOpts::test_defaults();
            run_checks_discovery_init_subprocess(work, &shared).expect("skip spawn");
        });
    }
}
