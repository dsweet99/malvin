use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::ModalInvocation;
use super::bridge::ModalBridge;
use super::config::{ModalConfig, remote_config_text};
use super::image::ImageSpec;
use super::remote::RemotePaths;
use super::workspace::{RECENT_RUN_DIRS, pack_logs, pack_workspace, recent_run_dirs};

#[path = "session_drive.rs"]
mod drive;

const TIMEOUT_HINT_MARGIN_S: u64 = 5;

pub struct Plan {
    pub cfg: ModalConfig,
    pub cfg_root: toml::Value,
    pub spec: ImageSpec,
}

pub struct Prepared {
    pub staging: tempfile::TempDir,
    pub cwd: PathBuf,
    pub home: PathBuf,
    pub logs_root: PathBuf,
    pub paths: RemotePaths,
    pub has_logs: bool,
    pub plan: Plan,
}

pub(super) fn note(text: &str) {
    super::remote_output::print_status(&format!("modal: {text}"));
}

pub fn prepare(cwd: &Path, home: &Path, plan: Plan) -> Result<Prepared, String> {
    let staging = tempfile::tempdir().map_err(|e| format!("create staging dir: {e}"))?;
    let stage = staging.path();
    pack_workspace(cwd, &stage.join("workspace.tar.gz"))?;
    let logs_root = crate::malvin_logs_root(cwd);
    let names = recent_run_dirs(&logs_root, RECENT_RUN_DIRS);
    let has_logs = pack_logs(&logs_root, &names, &stage.join("logs.tar.gz"))?;
    std::fs::write(
        stage.join("config.toml"),
        remote_config_text(&plan.cfg_root, &plan.cfg)?,
    )
    .map_err(|e| format!("write staged config.toml: {e}"))?;
    Ok(Prepared {
        paths: RemotePaths::new(cwd, home),
        staging,
        cwd: cwd.to_path_buf(),
        home: home.to_path_buf(),
        logs_root,
        has_logs,
        plan,
    })
}

fn sandbox_request(plan: &Plan, tags: &BTreeMap<String, String>) -> Value {
    json!({
        "image": plan.spec.name,
        "gpu": plan.cfg.gpu,
        "cpu": plan.cfg.ncpu,
        "memory_mib": plan.cfg.memory_gb * 1024,
        "timeout_ms": plan.cfg.timeout_s * 1000,
        "tags": tags,
    })
}

pub(super) fn image_rebuild_may_help(err: &str) -> bool {
    !err.contains("INVALID_ARGUMENT")
}

fn start_sandbox(
    bridge: &mut ModalBridge,
    plan: &Plan,
    tags: &BTreeMap<String, String>,
) -> Result<String, String> {
    let req = sandbox_request(plan, tags);
    let reply = match bridge.call("create_sandbox", req.clone()) {
        Ok(reply) => reply,
        Err(first) if !image_rebuild_may_help(&first) => return Err(first),
        Err(first) => {
            note(&format!("{first}; rebuilding the image and retrying once"));
            bridge.call("ensure_image", plan.spec.request(true))?;
            bridge.call("create_sandbox", req)?
        }
    };
    Ok(reply["sandbox_id"].as_str().unwrap_or("?").to_string())
}

pub(super) fn with_timeout_hint(
    err: String,
    elapsed: std::time::Duration,
    timeout_s: u64,
) -> String {
    if elapsed.as_secs() + TIMEOUT_HINT_MARGIN_S < timeout_s {
        return err;
    }
    format!(
        "{err}\nThe Sandbox likely reached its {timeout_s} s timeout; raise it with `--remote=modal:sandbox[timeout=...]` or `[modal] timeout` in config.toml"
    )
}

pub fn run(
    bridge: &mut ModalBridge,
    prep: &Prepared,
    inv: &ModalInvocation,
) -> Result<i32, String> {
    let host = super::sweep::host_name();
    let tags = super::sweep::run_tags(&crate::alnum_id::random_alnum(8), &host, std::process::id());
    let sandbox_id = start_sandbox(bridge, &prep.plan, &tags)?;
    note(&format!(
        "Sandbox {sandbox_id} started ({})",
        prep.plan.cfg.describe()
    ));
    let started = std::time::Instant::now();
    let result = drive::drive(bridge, prep, inv)
        .map_err(|e| with_timeout_hint(e, started.elapsed(), prep.plan.cfg.timeout_s));
    if !bridge.exited()
        && let Err(e) = bridge.call("terminate", json!({}))
    {
        note(&format!(
            "final terminate request for Sandbox {sandbox_id} failed ({e}); the bridge terminates its Sandbox when it exits"
        ));
    }
    let code = result?;
    drive::finish_locally(prep)?;
    Ok(code)
}
