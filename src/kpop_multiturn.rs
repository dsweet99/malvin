use std::path::PathBuf;

use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::kpop_acp_prompt::kpop_creative_enabled;
use crate::kpop_multiturn_prompts::KpopMultiturnPrompts;
use crate::multiturn_prompt::MultiturnPrompt;
use crate::kpop_schedule::{
    KPOP_CATCHUP_CAP, agent_declared_success, block_mean_from_p_creative, count_mbc2_entries,
    hypotheses_emitted, poisson_block_size, read_exp_log_text,
};

enum Phase {
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

enum NextStep {
    Stop,
    Again,
    Emit(MultiturnPrompt),
}

pub struct KpopMultiturnParams<B> {
    pub builder: B,
    pub exp_log_path: PathBuf,
    pub max_hypotheses: usize,
    pub p_creative: f64,
    pub rng: StdRng,
}

pub struct KpopMultiturnState<B: KpopMultiturnPrompts> {
    builder: B,
    exp_log_path: PathBuf,
    pub max_hypotheses: usize,
    pub p_creative: f64,
    rng: StdRng,
    credit: usize,
    phase: Phase,
    done: bool,
}

impl<B: KpopMultiturnPrompts> KpopMultiturnState<B> {
    pub fn exp_log_path(&self) -> &std::path::Path {
        &self.exp_log_path
    }

    pub fn new(builder: B, exp_log_path: PathBuf, max_hypotheses: usize, p_creative: f64) -> Result<Self, String> {
        Self::from_params(KpopMultiturnParams {
            builder,
            exp_log_path,
            max_hypotheses,
            p_creative,
            rng: StdRng::from_entropy(),
        })
    }

    pub fn from_params(mut params: KpopMultiturnParams<B>) -> Result<Self, String> {
        let text = read_exp_log_text(&params.exp_log_path)?;
        let hypotheses_before = hypotheses_emitted(&text);
        let mean = block_mean_from_p_creative(params.p_creative);
        let n = poisson_block_size(&mut params.rng, mean).max(1);
        let phase = Phase::KpopBlock {
            target_n: n,
            hypotheses_before,
            attempts: 0,
        };
        Ok(Self {
            builder: params.builder,
            exp_log_path: params.exp_log_path,
            max_hypotheses: params.max_hypotheses,
            p_creative: params.p_creative,
            rng: params.rng,
            credit: 0,
            phase,
            done: false,
        })
    }

    pub fn next_prompt(&mut self) -> Result<Option<MultiturnPrompt>, String> {
        if self.done {
            return Ok(None);
        }
        let text = read_exp_log_text(&self.exp_log_path)?;
        if agent_declared_success(&text) {
            self.done = true;
            return Ok(None);
        }
        if hypotheses_emitted(&text) >= self.max_hypotheses {
            self.done = true;
            return Ok(None);
        }

        let step = match &mut self.phase {
            Phase::KpopBlock { .. } => self.run_kpop_phase(&text)?,
            Phase::Mbc2 { .. } => self.run_mbc2_phase(&text)?,
        };
        match step {
            NextStep::Stop => Ok(None),
            NextStep::Again => self.next_prompt(),
            NextStep::Emit(s) => Ok(Some(s)),
        }
    }

    pub const fn record_kpop_block_prompt_completed(&mut self) {
        if let Phase::KpopBlock { attempts, .. } = &mut self.phase {
            *attempts += 1;
        }
    }

    pub const fn record_mbc2_prompt_completed(&mut self) {
        if let Phase::Mbc2 { sent, .. } = &mut self.phase {
            *sent = match *sent {
                0 => 1,
                _ => 2,
            };
        }
    }

    fn run_kpop_phase(&mut self, text: &str) -> Result<NextStep, String> {
        let hypotheses_now = hypotheses_emitted(text);
        let (need, hb, tn) = {
            let Phase::KpopBlock {
                target_n,
                hypotheses_before,
                attempts: _,
            } = &self.phase
            else {
                return Err("internal: expected KpopBlock phase".to_string());
            };
            let done_in_block = hypotheses_now.saturating_sub(*hypotheses_before);
            let w = target_n.saturating_sub(done_in_block);
            (w, *hypotheses_before, *target_n)
        };
        if need == 0 {
            return self.complete_kpop_block(hypotheses_now, hb, tn);
        }
        let Phase::KpopBlock {
            target_n: _,
            hypotheses_before: _,
            attempts,
        } = &mut self.phase
        else {
            return Err("internal: expected KpopBlock phase".to_string());
        };
        if *attempts > KPOP_CATCHUP_CAP {
            return Err(format!(
                "KPOP block still incomplete after the initial attempt and {KPOP_CATCHUP_CAP} catch-up attempts.",
            ));
        }
        let remaining_budget = self
            .max_hypotheses
            .saturating_sub(hypotheses_emitted(text));
        let want = need.min(remaining_budget);
        if want == 0 {
            self.done = true;
            return Ok(NextStep::Stop);
        }
        let remaining_after = remaining_budget.saturating_sub(want);
        self.builder
            .kpop_block(want, remaining_after)
            .map(|s| NextStep::Emit(MultiturnPrompt::KpopBlock(s)))
    }

    fn complete_kpop_block(
        &mut self,
        hypotheses_now: usize,
        hb: usize,
        tn: usize,
    ) -> Result<NextStep, String> {
        let actual = hypotheses_now.saturating_sub(hb);
        self.credit = actual.saturating_sub(tn);
        if !kpop_creative_enabled(self.p_creative) {
            let mean = block_mean_from_p_creative(self.p_creative);
            let n = self.credit + poisson_block_size(&mut self.rng, mean);
            self.credit = 0;
            self.phase = Phase::KpopBlock {
                target_n: n.max(1),
                hypotheses_before: hypotheses_now,
                attempts: 0,
            };
            return Ok(NextStep::Again);
        }
        let text = read_exp_log_text(&self.exp_log_path)?;
        let mbc2_before = count_mbc2_entries(&text);
        self.phase = Phase::Mbc2 {
            baseline: mbc2_before,
            sent: 0,
        };
        Ok(NextStep::Again)
    }

    fn run_mbc2_phase(&mut self, text: &str) -> Result<NextStep, String> {
        let Phase::Mbc2 { baseline, sent } = &mut self.phase else {
            return Err("internal: expected Mbc2 phase".to_string());
        };
        let m = count_mbc2_entries(text);
        if m > *baseline {
            self.start_new_block_after_mbc2()?;
            return Ok(NextStep::Again);
        }
        if *sent < 2 {
            return self
                .builder
                .mbc2_pure()
                .map(|s| NextStep::Emit(MultiturnPrompt::Mbc2(s)));
        }
        self.start_new_block_after_mbc2()?;
        Ok(NextStep::Again)
    }

    fn start_new_block_after_mbc2(&mut self) -> Result<(), String> {
        let text = read_exp_log_text(&self.exp_log_path)?;
        let hypotheses_before = hypotheses_emitted(&text);
        let mean = block_mean_from_p_creative(self.p_creative);
        let n = self.credit + poisson_block_size(&mut self.rng, mean);
        self.credit = 0;
        self.phase = Phase::KpopBlock {
            target_n: n.max(1),
            hypotheses_before,
            attempts: 0,
        };
        Ok(())
    }
}

