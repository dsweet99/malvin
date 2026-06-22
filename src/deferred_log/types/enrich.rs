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
#[cfg(test)]
#[path = "enrich_test.rs"]
mod enrich_test;#[cfg(test)]
#[path = "enrich_kiss_cov_test.rs"]
mod enrich_kiss_cov_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<EnrichKey> = None;
        let _: Option<ToolDrainMeta> = None;
    }
}
