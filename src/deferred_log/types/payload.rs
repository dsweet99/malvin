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
    Heartbeat {
        log_line: String,
    },
    TaggedStdout {
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
