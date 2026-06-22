use crate::tool_summary::LineRange;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolCallArgs {
    pub path: Option<String>,
    pub line_range: Option<LineRange>,
}
#[cfg(test)]
#[path = "types_test.rs"]
mod types_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<ToolCallArgs> = None;
    }
}
