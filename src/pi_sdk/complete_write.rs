use std::sync::Arc;

use async_trait::async_trait;
use pi::jobs::JobSessionScope;
use pi::sdk::{Tool, ToolOutput, ToolUpdate};
use pi::tools::{ToolEffects, ToolOrigin};
use serde_json::Value;

use super::super::tool_args_normalize::bind_shared_job_scope;

pub(super) struct CompleteWrite {
    inner: Arc<dyn Tool>,
}

impl CompleteWrite {
    pub(super) fn from_builtin(inner: Arc<dyn Tool>) -> Self {
        Self { inner }
    }
}

fn write_field_str<'a>(input: &'a Value, keys: &[&str]) -> &'a str {
    keys.iter()
        .find_map(|k| input.get(*k).and_then(Value::as_str))
        .unwrap_or("")
}

fn path_looks_like_bin(path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    normalized.contains("/bin/") || normalized.starts_with("bin/")
}

pub(super) fn stubby_write_error(input: &Value) -> Option<String> {
    let path = write_field_str(input, &["path", "file"]);
    if !path_looks_like_bin(path) {
        return None;
    }
    let trimmed = write_field_str(input, &["content", "text", "contents"]).trim();
    if trimmed.len() < 120 {
        return Some(
            "write of a bin/ script is too short; include the complete file body, not a stub"
                .to_string(),
        );
    }
    if trimmed.contains("...") && trimmed.lines().count() < 12 {
        return Some(
            "write of a bin/ script looks stubbed; include the complete file body".to_string(),
        );
    }
    None
}

fn push_escaped(out: &mut String, next: Option<char>) -> bool {
    match next {
        Some('n') => out.push('\n'),
        Some('t') => out.push('\t'),
        Some('\\') => out.push('\\'),
        Some('"') => out.push('"'),
        _ => {
            out.push('\\');
            return false;
        }
    }
    true
}

pub(super) fn unescape_local_write_content(content: &str) -> String {
    if content.contains('\n') || content.contains('\r') {
        return content.to_string();
    }
    if !(content.contains("\\n") || content.contains("\\t")) {
        return content.to_string();
    }
    let mut out = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            let next = chars.peek().copied();
            if push_escaped(&mut out, next) {
                chars.next();
            }
        } else {
            out.push(c);
        }
    }
    out
}

pub(super) fn normalize_write_input(mut input: Value) -> Value {
    let Some(obj) = input.as_object_mut() else {
        return input;
    };
    for key in ["content", "text", "contents"] {
        if let Some(Value::String(s)) = obj.get(key).cloned() {
            let fixed = unescape_local_write_content(&s);
            if fixed != s {
                obj.insert(key.to_string(), Value::String(fixed));
            }
            break;
        }
    }
    Value::Object(obj.clone())
}

#[async_trait]
impl Tool for CompleteWrite {
    fn name(&self) -> &'static str {
        "write"
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
        bind_shared_job_scope(&mut self.inner, scope);
    }

    fn origin(&self) -> ToolOrigin {
        self.inner.origin()
    }

    async fn execute(
        &self,
        tool_call_id: &str,
        input: Value,
        on_update: Option<Box<dyn Fn(ToolUpdate) + Send + Sync>>,
    ) -> pi::sdk::Result<ToolOutput> {
        let input = normalize_write_input(input);
        if let Some(msg) = stubby_write_error(&input) {
            return Err(pi::error::Error::validation(msg));
        }
        self.inner.execute(tool_call_id, input, on_update).await
    }
}

