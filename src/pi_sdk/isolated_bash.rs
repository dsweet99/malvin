use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use pi::sdk::{
    Config, Tool, ToolFactory, ToolOutput, ToolRegistry, ToolUpdate, default_tool_registry,
};
use pi::tools::ToolEffects;
use serde_json::Value;

pub(crate) struct IsolatedToolFactory;

impl ToolFactory for IsolatedToolFactory {
    fn create_tool_registry(&self, enabled: &[&str], cwd: &Path, config: &Config) -> ToolRegistry {
        let registry = default_tool_registry(enabled, cwd, config);
        let tools = registry.into_tools();
        let mut replaced = Vec::with_capacity(tools.len());
        for tool in tools {
            if tool.name() == "bash" {
                replaced.push(
                    Box::new(IsolatedBash::from_builtin(tool, cwd.to_path_buf())) as Box<dyn Tool>,
                );
            } else {
                replaced.push(tool);
            }
        }
        ToolRegistry::from_tools(replaced)
    }
}

struct IsolatedBash {
    inner: Box<dyn Tool>,
    cwd: PathBuf,
}

impl IsolatedBash {
    fn from_builtin(inner: Box<dyn Tool>, cwd: PathBuf) -> Self {
        Self { inner, cwd }
    }
}

#[async_trait]
impl Tool for IsolatedBash {
    fn name(&self) -> &'static str {
        "bash"
    }

    fn label(&self) -> &str {
        self.inner.label()
    }

    fn description(&self) -> &str {
        self.inner.description()
    }

    fn parameters(&self) -> Value {
        self.inner.parameters()
    }

    fn effects(&self) -> ToolEffects {
        ToolEffects::process().union(ToolEffects::write())
    }

    #[allow(clippy::unused_async)]
    async fn execute(
        &self,
        _tool_call_id: &str,
        input: Value,
        on_update: Option<Box<dyn Fn(ToolUpdate) + Send + Sync>>,
    ) -> pi::sdk::Result<ToolOutput> {
        let command = input
            .get("command")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let timeout_secs = input.get("timeout").and_then(Value::as_u64);
        run_isolated_bash(&self.cwd, &command, timeout_secs, on_update.as_deref())
    }
}

fn spawn_isolated_shell(cwd: &Path, command: &str) -> pi::sdk::Result<std::process::Child> {
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

fn run_isolated_bash(
    cwd: &Path,
    command: &str,
    timeout_secs: Option<u64>,
    on_update: Option<&(dyn Fn(ToolUpdate) + Send + Sync)>,
) -> pi::sdk::Result<ToolOutput> {
    let timeout = match timeout_secs {
        None => Some(Duration::from_mins(2)),
        Some(0) => None,
        Some(secs) => Some(Duration::from_secs(secs)),
    };
    let child = spawn_isolated_shell(cwd, command)?;
    #[cfg(unix)]
    crate::acp::note_session_affiliated_pid(child.id());
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

fn isolated_shell() -> &'static str {
    for path in ["/bin/bash", "/usr/bin/bash", "/usr/local/bin/bash"] {
        if Path::new(path).exists() {
            return path;
        }
    }
    "sh"
}

fn wait_isolated_output(
    mut child: std::process::Child,
    timeout: Option<Duration>,
) -> pi::sdk::Result<std::process::Output> {
    let Some(limit) = timeout else {
        return child
            .wait_with_output()
            .map_err(|e| pi::error::Error::tool("bash", format!("isolated bash wait: {e}")));
    };
    let started = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return read_reaped_output(child, status),
            Ok(None) if started.elapsed() < limit => {
                std::thread::sleep(Duration::from_millis(20));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(pi::error::Error::tool(
                    "bash",
                    format!("isolated bash timed out after {limit:?}"),
                ));
            }
            Err(e) => {
                return Err(pi::error::Error::tool(
                    "bash",
                    format!("isolated bash wait: {e}"),
                ));
            }
        }
    }
}

fn read_reaped_output(
    mut child: std::process::Child,
    status: std::process::ExitStatus,
) -> pi::sdk::Result<std::process::Output> {
    use std::io::Read;

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    if let Some(mut pipe) = child.stdout.take() {
        pipe.read_to_end(&mut stdout)
            .map_err(|e| pi::error::Error::tool("bash", format!("isolated bash stdout: {e}")))?;
    }
    if let Some(mut pipe) = child.stderr.take() {
        pipe.read_to_end(&mut stderr)
            .map_err(|e| pi::error::Error::tool("bash", format!("isolated bash stderr: {e}")))?;
    }
    Ok(std::process::Output {
        status,
        stdout,
        stderr,
    })
}

#[must_use]
pub(crate) fn isolated_tool_factory() -> Arc<dyn ToolFactory> {
    Arc::new(IsolatedToolFactory)
}

#[cfg(test)]
mod tests {
    use super::isolated_shell;

    #[test]
    fn isolated_shell_is_nonempty() {
        assert!(!isolated_shell().is_empty());
    }
}
