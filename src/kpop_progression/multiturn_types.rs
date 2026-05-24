use std::path::PathBuf;

use rand::rngs::StdRng;

use crate::kpop_multiturn_prompts::KpopMultiturnPrompts;
use crate::multiturn_prompt::MultiturnPrompt;

pub(crate) enum Phase {
    KpopBlock {
        target_n: usize,
        hypotheses_before: usize,
        attempts: u32,
    },
    Mbc2 {
        baseline: usize,
        sent: u32,
    },
}

pub(crate) enum NextStep {
    Stop,
    Again,
    Emit(MultiturnPrompt),
}

pub struct KpopMultiturnParams<'a> {
    pub builder: KpopMultiturnPrompts<'a>,
    pub exp_log_path: PathBuf,
    pub max_hypotheses: usize,
    pub p_creative: f64,
    pub rng: StdRng,
}
