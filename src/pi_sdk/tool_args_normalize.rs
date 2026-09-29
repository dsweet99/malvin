use async_trait::async_trait;
use pi::sdk::{Tool, ToolOutput, ToolUpdate};
use pi::tools::ToolEffects;
use serde_json::Value;

const PATH_TOOLS: [&str; 5] = ["read", "write", "edit", "ls", "find"];
const PATH_ALIASES: [&str; 3] = ["file", "file_path", "filePath"];

pub(super) fn wrap_tool_args(tool: Box<dyn Tool>) -> Box<dyn Tool> {
    let path_aliases = PATH_TOOLS.contains(&tool.name());
    let schema = tool.parameters();
    Box::new(NormalizedArgsTool {
        inner: tool,
        schema,
        path_aliases,
    })
}

pub(super) fn normalize_path_alias(mut input: Value) -> Value {
    let Some(obj) = input.as_object_mut() else {
        return input;
    };
    if obj.contains_key("path") {
        return input;
    }
    if let Some(v) = PATH_ALIASES.iter().find_map(|k| obj.remove(*k)) {
        obj.insert("path".to_string(), v);
    }
    input
}

fn coerce_scalar(declared: &str, s: &str) -> Option<Value> {
    let s = s.trim();
    match declared {
        "boolean" if s.eq_ignore_ascii_case("true") => Some(Value::Bool(true)),
        "boolean" if s.eq_ignore_ascii_case("false") => Some(Value::Bool(false)),
        "integer" => s.parse::<i64>().ok().map(Value::from),
        _ => None,
    }
}

pub(super) fn coerce_string_scalars(mut input: Value, schema: &Value) -> Value {
    let Some(obj) = input.as_object_mut() else {
        return input;
    };
    for (key, value) in obj.iter_mut() {
        let declared = schema["properties"][key.as_str()]["type"].as_str();
        let fixed = match (declared, value.as_str()) {
            (Some(d), Some(s)) => coerce_scalar(d, s),
            _ => None,
        };
        if let Some(v) = fixed {
            *value = v;
        }
    }
    input
}

struct NormalizedArgsTool {
    inner: Box<dyn Tool>,
    schema: Value,
    path_aliases: bool,
}

#[async_trait]
impl Tool for NormalizedArgsTool {
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

    async fn execute(
        &self,
        tool_call_id: &str,
        input: Value,
        on_update: Option<Box<dyn Fn(ToolUpdate) + Send + Sync>>,
    ) -> pi::sdk::Result<ToolOutput> {
        let input = if self.path_aliases {
            normalize_path_alias(input)
        } else {
            input
        };
        let input = coerce_string_scalars(input, &self.schema);
        self.inner.execute(tool_call_id, input, on_update).await
    }
}

#[cfg(test)]
#[path = "tool_args_normalize_tests.rs"]
mod tests;
