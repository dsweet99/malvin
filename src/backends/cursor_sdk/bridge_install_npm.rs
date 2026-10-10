use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const NPM_CI_ARGS: &[&str] = &[
    "ci",
    "--omit=dev",
    "--no-audit",
    "--no-fund",
    "--prefer-offline",
];

pub(crate) fn npm_ci(dest: &Path, label: &str) -> Result<(), String> {
    let node = super::cursor_sdk::node_resolve::resolve_node_bin()?;
    let npm = resolve_npm(&node)?;
    crate::output::print_stderr_line(
        crate::output::MALVIN_WHO,
        &format!("installing {label} into {} with npm ci…", dest.display()),
    );
    let status = npm_command(&npm, &node, dest)
        .status()
        .map_err(|e| format!("failed to run {}: {e}", npm.display()))?;
    if status.success() {
        return Ok(());
    }
    Err(format!(
        "`{} {}` failed in {} ({status}); check network access to the npm registry",
        npm.display(),
        NPM_CI_ARGS.join(" "),
        dest.display()
    ))
}

pub(crate) fn resolve_npm(node: &Path) -> Result<PathBuf, String> {
    if let Some(p) = std::env::var_os("MALVIN_NPM").filter(|v| !v.is_empty()) {
        return Ok(PathBuf::from(p));
    }
    if let Some(sibling) = node.parent().map(|d| d.join("npm")).filter(|p| p.is_file()) {
        return Ok(sibling);
    }
    crate::workspace::support_paths::lookup_bin_on_path("npm").ok_or_else(|| {
        format!(
            "npm not found next to {} or on PATH (set MALVIN_NPM)",
            node.display()
        )
    })
}

pub(crate) fn path_with_node_first(node: &Path) -> OsString {
    let mut dirs: Vec<PathBuf> = node.parent().map(Path::to_path_buf).into_iter().collect();
    if let Some(path) = std::env::var_os("PATH") {
        dirs.extend(std::env::split_paths(&path));
    }
    std::env::join_paths(dirs).unwrap_or_default()
}

fn npm_command(npm: &Path, node: &Path, dest: &Path) -> Command {
    let mut cmd = Command::new(npm);
    cmd.args(NPM_CI_ARGS)
        .current_dir(dest)
        .env("PATH", path_with_node_first(node))
        .stdin(Stdio::null());
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
