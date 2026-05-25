#[derive(Clone, Debug)]
pub struct TeeSinkMeta {
    pub who: String,
    pub ts: String,
    pub emit_stdout_markdown: bool,
}

#[derive(Clone, Debug)]
pub struct ToolSummaryBuild {
    pub tee: TeeSinkMeta,
    pub plain: String,
    pub display: String,
    pub enrich: Option<super::EnrichKey>,
    pub meta: Option<super::ToolDrainMeta>,
}

#[derive(Clone, Debug)]
pub struct AcpTeeBuild {
    pub tee: TeeSinkMeta,
    pub kind: Option<crate::acp::SessionUpdateChunkKind>,
    pub line: String,
    pub display: Option<String>,
    pub dim_payload: bool,
}
