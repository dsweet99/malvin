use crate::acp::SessionUpdateChunkKind;

use super::{EnrichKey, ToolDrainMeta};

#[derive(Clone, Debug)]
pub enum DeferredPayload {
    ToolSummary {
        plain: String,
        display: String,
        enrich: Option<EnrichKey>,
        meta: Option<ToolDrainMeta>,
    },
    AcpTee {
        line: String,
        display: Option<String>,
        dim_payload: bool,
    },
    RawLine {
        line: String,
    },
    DisplayLog {
        display: String,
        log: String,
    },
}

#[derive(Clone, Debug)]
pub struct DeferredEntry {
    pub enqueued_at: std::time::Instant,
    pub who: String,
    pub ts: String,
    pub emit_stdout_markdown: bool,
    #[allow(dead_code)]
    pub kind: Option<SessionUpdateChunkKind>,
    pub payload: DeferredPayload,
}
#[cfg(test)]
#[path = "payload_test.rs"]
mod payload_test;#[cfg(test)]
#[path = "payload_kiss_cov_test.rs"]
mod payload_kiss_cov_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<DeferredEntry> = None;
        let _: Option<DeferredPayload> = None;
    }
}
