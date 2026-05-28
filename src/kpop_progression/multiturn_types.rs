use std::path::PathBuf;

use crate::kpop_multiturn_prompts::KpopMultiturnPrompts;
pub struct KpopMultiturnParams<'a> {
    pub builder: KpopMultiturnPrompts<'a>,
    pub exp_log_path: PathBuf,
    pub max_hypotheses: usize,
}
