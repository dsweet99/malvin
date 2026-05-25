use crate::tool_summary::LineRange;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolCallArgs {
    pub path: Option<String>,
    pub line_range: Option<LineRange>,
}
