mod backend_setup;
mod backends;
mod bridge;
pub mod config;
mod credentials;
pub mod gpu_types;
mod image;
pub mod options;
mod remote;
mod remote_output;
mod results;
mod session;
mod sweep;
mod workspace;

use std::ffi::OsString;
use std::path::PathBuf;

pub const REMOTE_FLAG: &str = "--remote";
pub const MODAL_REMOTE: &str = "modal";
pub const MODAL_SERVICE: &str = "sandbox";
pub const MODAL_REMOTE_ID: &str = "modal:sandbox";
pub const REMOTE_MODAL_ARG: &str = "--remote=modal:sandbox";
pub const MODAL_ABOUT: &str =
    "Run in a Modal Sandbox, then apply its file changes and copy its run logs here";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModalInvocation {
    pub remote_args: Vec<OsString>,
    pub model: String,
    pub request_files: Vec<PathBuf>,
    pub options: options::ModalOptions,
}

#[must_use]
pub fn remote_args(raw: &[OsString]) -> Vec<OsString> {
    raw.iter()
        .skip(1)
        .filter(|a| a.as_os_str() != REMOTE_MODAL_ARG)
        .cloned()
        .collect()
}

const UNSUPPORTED: &[(&str, &str)] = &[
    (
        "--watch",
        "the request is uploaded once, so later local edits would not reach the Sandbox",
    ),
    (
        "--iml",
        "results come back only when the remote run ends, and `--iml` never ends",
    ),
];

pub fn reject_unsupported(set_flags: &[&str]) -> Result<(), String> {
    for (flag, reason) in UNSUPPORTED {
        if set_flags.contains(flag) {
            return Err(format!(
                "`{REMOTE_MODAL_ARG}` cannot be combined with `{flag}`: {reason}"
            ));
        }
    }
    Ok(())
}

fn announce_sweep(result: Result<usize, String>) {
    match result {
        Ok(0) => {}
        Ok(n) => session::note(&format!(
            "terminated {n} stale Sandbox(es) left by exited malvin runs"
        )),
        Err(e) => session::note(&format!("stale Sandbox check failed; continuing: {e}")),
    }
}

fn load_plan(cwd: &std::path::Path, inv: &ModalInvocation) -> Result<session::Plan, String> {
    let cfg_root = config::read_config_root(&crate::malvin_home_config_path())?;
    let cfg = config::parse_modal_config(
        &cfg_root,
        crate::config::mem_limit_config::load_mem_limit_gb(cwd),
    )?
    .with_options(&inv.options);
    let spec = image::image_spec(&cfg, &inv.model, image::uploadable_binary())?;
    Ok(session::Plan {
        cfg,
        cfg_root,
        spec,
    })
}

fn start_bridge() -> Result<bridge::ModalBridge, String> {
    let bridge_js = bridge::ensure_installed()?;
    let cmd = bridge::node_bridge_command(&bridge_js).map_err(|e| {
        format!("malvin --remote=modal:sandbox needs Node.js >= 22.13 to run the Modal bridge: {e}")
    })?;
    bridge::ModalBridge::spawn(cmd)
}

pub fn run_modal(inv: &ModalInvocation) -> Result<i32, String> {
    let home = crate::user_home_dir();
    backend_setup::preflight(&inv.model, &home)?;
    let cwd = crate::canonical_work_dir_for_logs(
        &std::env::current_dir().map_err(|e| format!("current dir: {e}"))?,
    );
    workspace::check_upload_size(&cwd, workspace::MAX_NON_GIT_UPLOAD_BYTES)?;
    let plan = load_plan(&cwd, inv)?;
    let mut bridge = start_bridge()?;
    announce_sweep(sweep::sweep_stale(&mut bridge, &sweep::host_name()));
    bridge.call("ensure_image", plan.spec.request(false))?;
    let prep = session::prepare(&cwd, &home, plan)?;
    session::run(&mut bridge, &prep, inv)
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_gpu_types;
#[cfg(test)]
mod tests_io;
#[cfg(test)]
mod tests_options;
#[cfg(test)]
mod tests_remote;
