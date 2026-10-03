use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use pi::jobs::JobSessionScope;
use pi::model::ContentBlock;
use pi::sdk::{Tool, ToolOutput, ToolUpdate};
use pi::tools::{ToolEffects, ToolOrigin};
use serde_json::Value;

use super::super::tool_args_normalize::bind_shared_job_scope;

pub(super) const CWD_PINNED_TOOLS: &[&str] = &["edit", "hashline_edit", "ls", "grep", "find"];

pub(super) struct OutsideCwd {
    inner: Arc<dyn Tool>,
    rooted: Arc<dyn Tool>,
    cwd: PathBuf,
}

impl OutsideCwd {
    pub(super) fn new(inner: Arc<dyn Tool>, rooted: Arc<dyn Tool>, cwd: PathBuf) -> Self {
        Self { inner, rooted, cwd }
    }
}

fn expand_home(raw: &str) -> PathBuf {
    match (raw.strip_prefix("~/"), std::env::var_os("HOME")) {
        (Some(rest), Some(home)) => PathBuf::from(home).join(rest),
        _ if raw == "~" => std::env::var_os("HOME").map_or_else(|| raw.into(), PathBuf::from),
        _ => PathBuf::from(raw),
    }
}

fn lexical_normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other),
        }
    }
    out
}

fn canonical_or_lexical(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| lexical_normalize(path))
}

pub(super) fn outside_cwd_path(input: &Value, cwd: &Path) -> Option<String> {
    let raw = input.get("path").and_then(Value::as_str)?.trim();
    if raw.is_empty() {
        return None;
    }
    let expanded = expand_home(raw);
    let absolute = if expanded.is_absolute() {
        expanded
    } else {
        cwd.join(expanded)
    };
    let resolved = canonical_or_lexical(&absolute);
    if resolved.starts_with(canonical_or_lexical(cwd)) {
        return None;
    }
    Some(resolved.to_string_lossy().into_owned())
}

#[async_trait]
impl Tool for OutsideCwd {
    fn name(&self) -> &str {
        self.inner.name()
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
        self.inner.effects()
    }

    fn bind_job_session_scope(&mut self, scope: JobSessionScope) {
        bind_shared_job_scope(&mut self.inner, scope.clone());
        bind_shared_job_scope(&mut self.rooted, scope);
    }

    fn origin(&self) -> ToolOrigin {
        self.inner.origin()
    }

    async fn execute(
        &self,
        tool_call_id: &str,
        mut input: Value,
        on_update: Option<Box<dyn Fn(ToolUpdate) + Send + Sync>>,
    ) -> pi::sdk::Result<ToolOutput> {
        let Some(absolute) = outside_cwd_path(&input, &self.cwd) else {
            return self.inner.execute(tool_call_id, input, on_update).await;
        };
        input["path"] = Value::String(absolute.clone());
        let mut output = self.rooted.execute(tool_call_id, input, on_update).await?;
        if self.inner.name() == "grep" {
            absolutize_grep_paths(&mut output, &absolute);
        }
        Ok(output)
    }
}

pub(super) fn absolutize_grep_paths(output: &mut ToolOutput, absolute: &str) {
    let relative = absolute.trim_start_matches('/');
    if relative.is_empty() {
        return;
    }
    for block in &mut output.content {
        if let ContentBlock::Text(text) = block {
            text.text = text
                .text
                .split('\n')
                .map(|line| {
                    if line.starts_with(relative) {
                        format!("/{line}")
                    } else {
                        line.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
        }
    }
}
