use std::path::PathBuf;

use crate::kpop_multiturn_prompts::KpopMultiturnPrompts;
pub struct KpopMultiturnParams<'a> {
    pub builder: KpopMultiturnPrompts<'a>,
    pub exp_log_path: PathBuf,
    pub max_hypotheses: usize,
}
#[cfg(test)]
#[path = "multiturn_types_test.rs"]
mod multiturn_types_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<KpopMultiturnParams> = None;
    }
}
