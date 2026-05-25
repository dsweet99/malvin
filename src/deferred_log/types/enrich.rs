use serde_json::Value;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct EnrichKey {
    pub tool_call_id: String,
    #[allow(dead_code)]
    pub kind: String,
}

#[derive(Clone, Debug)]
pub struct ToolDrainMeta {
    pub tool_call_id: String,
    pub kind: String,
    pub elapsed: Duration,
    pub raw_output: Option<Value>,
    pub fallback_plain: String,
}
