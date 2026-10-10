use std::path::Path;

use serde_json::json;

use super::super::ModalInvocation;
use super::super::bridge::ModalBridge;
use super::super::remote;
use super::super::results::{apply_patch, describe_outcome, import_logs};
use super::{Prepared, note};

fn upload(bridge: &mut ModalBridge, local: &Path, remote: &str) -> Result<(), String> {
    bridge
        .call(
            "upload",
            json!({ "local": local.display().to_string(), "remote": remote }),
        )
        .map(|_| ())
}

fn upload_all(
    bridge: &mut ModalBridge,
    prep: &Prepared,
    inv: &ModalInvocation,
) -> Result<(), String> {
    let stage = prep.staging.path();
    upload(
        bridge,
        &stage.join("workspace.tar.gz"),
        remote::WORKSPACE_TAR,
    )?;
    if prep.has_logs {
        upload(bridge, &stage.join("logs.tar.gz"), remote::LOGS_TAR)?;
    }
    upload(
        bridge,
        &stage.join("config.toml"),
        &prep.paths.config_path(),
    )?;
    for file in super::super::backend_setup::setup_for(&inv.model)?.login_files(&prep.home) {
        note(&format!(
            "copying {} into the Sandbox for this run",
            file.display()
        ));
        upload(bridge, &file, &file.display().to_string())?;
    }
    for file in &inv.request_files {
        upload(
            bridge,
            &prep.cwd.join(file),
            &prep.cwd.join(file).display().to_string(),
        )?;
    }
    Ok(())
}

fn exec_script(bridge: &mut ModalBridge, script: &str, prep: &Prepared) -> Result<(), String> {
    let reply = bridge.call(
        "exec",
        json!({ "argv": ["sh", "-c", script], "env": prep.paths.script_env() }),
    )?;
    if reply["exit_code"].as_i64() == Some(0) {
        return Ok(());
    }
    Err(format!(
        "remote step failed (exit {}): {}{}",
        reply["exit_code"],
        reply["stdout"].as_str().unwrap_or_default(),
        reply["stderr"].as_str().unwrap_or_default()
    ))
}

fn run_malvin(
    bridge: &mut ModalBridge,
    prep: &Prepared,
    inv: &ModalInvocation,
) -> Result<i32, String> {
    let mut argv = vec!["malvin".to_string()];
    argv.extend(
        inv.remote_args
            .iter()
            .map(|a| a.to_string_lossy().into_owned()),
    );
    let mut forwarded = super::super::credentials::forwarded_env(|k| std::env::var(k).ok());
    forwarded.extend(super::super::backend_setup::setup_for(&inv.model)?.sandbox_env());
    let env = prep.paths.malvin_env(forwarded);
    let reply = bridge.call(
        "exec",
        json!({ "argv": argv, "workdir": prep.paths.work_dir, "env": env, "stream": true }),
    )?;
    let code = reply["exit_code"].as_i64().unwrap_or(1);
    Ok(i32::try_from(code).unwrap_or(1))
}

fn download(bridge: &mut ModalBridge, remote_path: &str, local: &Path) -> Result<(), String> {
    bridge
        .call(
            "download",
            json!({ "remote": remote_path, "local": local.display().to_string() }),
        )
        .map(|_| ())
}

pub(super) fn drive(
    bridge: &mut ModalBridge,
    prep: &Prepared,
    inv: &ModalInvocation,
) -> Result<i32, String> {
    upload_all(bridge, prep, inv)?;
    exec_script(bridge, remote::SETUP_SCRIPT, prep)?;
    note("workspace uploaded; starting the remote malvin");
    let code = run_malvin(bridge, prep, inv)?;
    exec_script(bridge, remote::FINISH_SCRIPT, prep)?;
    let stage = prep.staging.path();
    download(bridge, remote::RESULT_PATCH, &stage.join("result.patch"))?;
    download(bridge, remote::NEW_LOGS_TAR, &stage.join("new_logs.tar.gz"))?;
    Ok(code)
}

pub(super) fn finish_locally(prep: &Prepared) -> Result<(), String> {
    let stage = prep.staging.path();
    let imported = import_logs(&stage.join("new_logs.tar.gz"), &prep.logs_root)?;
    for dir in &imported {
        note(&format!("run logs copied to {}", dir.display()));
    }
    let keep_dir = imported.last().unwrap_or(&prep.logs_root);
    let outcome = apply_patch(&stage.join("result.patch"), &prep.cwd, keep_dir)?;
    note(&describe_outcome(&outcome));
    Ok(())
}
