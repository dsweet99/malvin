use super::model_id_spec::{BackendSpec, parsed};
use super::{CLAUDE_PREFIX, ModelBackend, ParsedModel};

pub(super) struct ClaudeSpec;

impl BackendSpec for ClaudeSpec {
    fn prefix(&self) -> &'static str {
        CLAUDE_PREFIX
    }
    fn label(&self) -> &'static str {
        "claude"
    }
    fn drain_idle_prefix(&self) -> &'static str {
        "claude timed out"
    }
    fn wire_label(&self) -> &'static str {
        "claude"
    }
    fn wire_parse_label(&self) -> Option<&'static str> {
        Some("JSONL")
    }
    fn wire_model(&self, model: &ParsedModel) -> String {
        model.slug.clone()
    }
    fn parse_slug(&self, rest: &str) -> Result<ParsedModel, String> {
        let model = parsed(ModelBackend::Claude, rest)?;
        if model.params.is_empty() {
            Ok(model)
        } else {
            Err("claude: model ids do not support bracket parameters".to_string())
        }
    }
}
