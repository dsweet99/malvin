use std::path::Path;
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

use pi::sdk::{ToolOutput, ToolUpdate};

#[cfg(unix)]
static ACTIVE_ISOLATED_BASH_PID: LazyLock<Mutex<Option<u32>>> =
    LazyLock::new(|| Mutex::new(None));

pub(crate) fn spawn_isolated_shell(cwd: &Path, command: &str) -> pi::sdk::Result<std::process::Child> {
    let shell = isolated_shell();
    let mut cmd = crate::malvin_sandbox::malvin_std_command(shell);
    cmd.arg("-c")
        .arg(command)
        .current_dir(cwd)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    cmd.spawn().map_err(|e| {
        pi::error::Error::tool("bash", format!("Failed to spawn isolated shell: {e}"))
    })
}

fn tool_text_output(text: String, output: &std::process::Output) -> ToolOutput {
    ToolOutput {
        content: vec![pi::model::ContentBlock::Text(pi::model::TextContent::new(
            text,
        ))],
        details: Some(serde_json::json!({
            "exitCode": output.status.code().unwrap_or(-1),
            "isolated": true,
        })),
        is_error: !output.status.success(),
    }
}

pub(crate) fn run_isolated_bash(
    cwd: &Path,
    command: &str,
    timeout_secs: Option<u64>,
    on_update: Option<&(dyn Fn(ToolUpdate) + Send + Sync)>,
) -> pi::sdk::Result<ToolOutput> {
    let timeout = match timeout_secs {
        None | Some(0) => Some(Duration::from_mins(2)),
        Some(secs) => Some(Duration::from_secs(secs)),
    };
    let child = spawn_isolated_shell(cwd, command)?;
    #[cfg(unix)]
    {
        let shell_pgid = child.id();
        crate::acp::note_session_affiliated_pid(shell_pgid);
        set_active_isolated_bash_pid(shell_pgid);
        let output = match wait_isolated_output(child, timeout) {
            Ok(output) => output,
            Err(err) => {
                clear_active_isolated_bash_pid();
                return Err(err);
            }
        };
        clear_active_isolated_bash_pid();
        reap_isolated_shell_process_group(shell_pgid);
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        if let Some(cb) = on_update {
            cb(ToolUpdate {
                content: vec![pi::model::ContentBlock::Text(pi::model::TextContent::new(
                    text.clone(),
                ))],
                details: None,
            });
        }
        Ok(tool_text_output(text, &output))
    }
    #[cfg(not(unix))]
    {
        let output = wait_isolated_output(child, timeout)?;
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        if let Some(cb) = on_update {
            cb(ToolUpdate {
                content: vec![pi::model::ContentBlock::Text(pi::model::TextContent::new(
                    text.clone(),
                ))],
                details: None,
            });
        }
        Ok(tool_text_output(text, &output))
    }
}

pub(crate) fn isolated_shell() -> &'static str {
    for path in ["/bin/bash", "/usr/bin/bash", "/usr/local/bin/bash"] {
        if Path::new(path).exists() {
            return path;
        }
    }
    "sh"
}

#[cfg(unix)]
fn set_active_isolated_bash_pid(pid: u32) {
    *ACTIVE_ISOLATED_BASH_PID
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(pid);
}

#[cfg(unix)]
fn clear_active_isolated_bash_pid() {
    *ACTIVE_ISOLATED_BASH_PID
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
}

pub(crate) fn interrupt_active_isolated_bash() {
    #[cfg(unix)]
    {
        let pid = ACTIVE_ISOLATED_BASH_PID
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        if let Some(pid) = pid {
            crate::acp::signal_process_group(pid, 9);
        }
    }
}

#[cfg(unix)]
pub(crate) fn reap_isolated_shell_process_group(shell_pgid: u32) {
    crate::acp::signal_process_group(shell_pgid, 9);
}

pub(crate) fn wait_isolated_output(
    child: std::process::Child,
    timeout: Option<Duration>,
) -> pi::sdk::Result<std::process::Output> {
    let Some(limit) = timeout else {
        let child = child;
        return child
            .wait_with_output()
            .map_err(|e| pi::error::Error::tool("bash", format!("isolated bash wait: {e}")));
    };
    crate::command_output_timeout::wait_piped_child_with_timeout(child, limit, "isolated bash")
        .map_err(|e| pi::error::Error::tool("bash", e))
}
