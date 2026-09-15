use std::path::Path;
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

use pi::sdk::{ToolOutput, ToolUpdate};

#[cfg(unix)]
static ACTIVE_ISOLATED_BASH_PID: LazyLock<Mutex<Option<u32>>> = LazyLock::new(|| Mutex::new(None));

pub(crate) fn spawn_isolated_shell(
    cwd: &Path,
    command: &str,
) -> pi::sdk::Result<std::process::Child> {
    let shell = isolated_shell();
    let command = rewrite_wget_downloads_to_curl(command);
    let mut cmd = crate::malvin_sandbox::malvin_std_command(shell);
    cmd.arg("-c")
        .arg(command)
        .current_dir(cwd)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    cmd.spawn()
        .map_err(|e| pi::error::Error::tool("bash", format!("Failed to spawn isolated shell: {e}")))
}

fn rewrite_wget_downloads_to_curl(command: &str) -> String {
    let trimmed = command.trim();
    let Some(rest) = trimmed.strip_prefix("wget") else {
        return command.to_string();
    };
    if !rest.starts_with(|c: char| c.is_whitespace()) && !rest.is_empty() {
        return command.to_string();
    }
    let tokens: Vec<&str> = rest.split_whitespace().collect();
    let mut out_path: Option<&str> = None;
    let mut url: Option<&str> = None;
    let mut i = 0;
    while i < tokens.len() {
        let tok = tokens[i];
        if tok == "-O" || tok == "--output-document" {
            if let Some(path) = tokens.get(i + 1) {
                out_path = Some(*path);
                i += 2;
                continue;
            }
        } else if let Some(path) = tok.strip_prefix("-O") {
            if !path.is_empty() {
                out_path = Some(path);
                i += 1;
                continue;
            }
        } else if tok.starts_with("http://") || tok.starts_with("https://") {
            url = Some(tok);
        }
        i += 1;
    }
    match (out_path, url) {
        (Some(path), Some(url)) => format!("curl -L --fail -o {path} {url}"),
        (None, Some(url)) => format!("curl -L --fail -O {url}"),
        _ => command.to_string(),
    }
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

#[cfg(test)]
mod rewrite_wget_tests {
    use super::rewrite_wget_downloads_to_curl;

    #[test]
    fn rewrite_wget_o_flag_to_curl() {
        assert_eq!(
            rewrite_wget_downloads_to_curl(
                "wget -O /tmp/arxiv.pdf https://arxiv.org/pdf/2506.12818"
            ),
            "curl -L --fail -o /tmp/arxiv.pdf https://arxiv.org/pdf/2506.12818"
        );
        assert_eq!(
            rewrite_wget_downloads_to_curl("wget -O/tmp/x.pdf https://example.com/a"),
            "curl -L --fail -o /tmp/x.pdf https://example.com/a"
        );
        assert_eq!(
            rewrite_wget_downloads_to_curl("curl -L -o /tmp/x https://example.com"),
            "curl -L -o /tmp/x https://example.com"
        );
        assert_eq!(
            rewrite_wget_downloads_to_curl("wget2 -O /tmp/x https://example.com"),
            "wget2 -O /tmp/x https://example.com"
        );
    }
}
