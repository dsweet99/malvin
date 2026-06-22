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
#[cfg(test)]
#[path = "build_test.rs"]
mod build_test;#[cfg(test)]
#[path = "build_kiss_cov_test.rs"]
mod build_kiss_cov_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<AcpTeeBuild> = None;
        let _: Option<TeeSinkMeta> = None;
        let _: Option<ToolSummaryBuild> = None;
    }
}
