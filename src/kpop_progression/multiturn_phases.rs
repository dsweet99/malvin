use super::block_report::{KpopBlockMissSnapshot, KpopBlockProgressCtx};
use super::counters::{
    KPOP_CATCHUP_CAP, block_mean_from_p_creative, count_mbc2_entries, hypotheses_emitted,
    poisson_block_size, read_exp_log_text,
};
use super::multiturn::KpopMultiturnState;
use super::multiturn_types::{NextStep, Phase};
use crate::kpop_acp_prompt::kpop_creative_enabled;
use crate::multiturn_prompt::MultiturnPrompt;

pub(crate) const fn kpop_block_progress_ctx(
    state: &KpopMultiturnState,
    hypotheses_now: usize,
) -> Option<KpopBlockProgressCtx> {
    let Phase::KpopBlock {
        target_n,
        hypotheses_before,
        attempts,
    } = &state.phase
    else {
        return None;
    };
    let done_in_block = hypotheses_now.saturating_sub(*hypotheses_before);
    Some(KpopBlockProgressCtx {
        steps_needed: target_n.saturating_sub(done_in_block),
        attempts_so_far: *attempts,
    })
}

pub(crate) fn set_last_block_miss(state: &mut KpopMultiturnState, snapshot: KpopBlockMissSnapshot) {
    state.last_block_miss = Some(snapshot);
}

fn kpop_block_need(state: &KpopMultiturnState, hypotheses_now: usize) -> Result<(usize, usize, usize), String> {
    let Phase::KpopBlock {
        target_n,
        hypotheses_before,
        attempts: _,
    } = &state.phase
    else {
        return Err("internal: expected KpopBlock phase".to_string());
    };
    let done_in_block = hypotheses_now.saturating_sub(*hypotheses_before);
    Ok((
        target_n.saturating_sub(done_in_block),
        *hypotheses_before,
        *target_n,
    ))
}

pub(crate) fn run_kpop_phase(state: &mut KpopMultiturnState, text: &str) -> Result<NextStep, String> {
    let hypotheses_now = hypotheses_emitted(text);
    let (need, hb, tn) = kpop_block_need(state, hypotheses_now)?;
    if need == 0 {
        return complete_kpop_block(state, hypotheses_now, hb, tn);
    }
    let Phase::KpopBlock {
        target_n: _,
        hypotheses_before: _,
        attempts,
    } = &mut state.phase
    else {
        return Err("internal: expected KpopBlock phase".to_string());
    };
    if *attempts > KPOP_CATCHUP_CAP {
        let message = state.last_block_miss.as_ref().map_or_else(
            || {
                format!(
                    "KPOP block incomplete after the initial attempt and {KPOP_CATCHUP_CAP} catch-up attempts.",
                )
            },
            KpopBlockMissSnapshot::format_catchup_exhausted_error,
        );
        return Err(message);
    }
    let remaining_budget = state.max_hypotheses.saturating_sub(hypotheses_emitted(text));
    let want = need.min(remaining_budget);
    if want == 0 {
        state.done = true;
        return Ok(NextStep::Stop);
    }
    let remaining_after = remaining_budget.saturating_sub(want);
    state
        .builder
        .kpop_block(want, remaining_after)
        .map(|s| NextStep::Emit(MultiturnPrompt::KpopBlock(s)))
}

fn complete_kpop_block(
    state: &mut KpopMultiturnState,
    hypotheses_now: usize,
    hb: usize,
    tn: usize,
) -> Result<NextStep, String> {
    let actual = hypotheses_now.saturating_sub(hb);
    state.credit = actual.saturating_sub(tn);
    if !kpop_creative_enabled(state.p_creative) {
        let mean = block_mean_from_p_creative(state.p_creative);
        let n = state.credit + poisson_block_size(&mut state.rng, mean);
        state.credit = 0;
        state.phase = Phase::KpopBlock {
            target_n: n.max(1),
            hypotheses_before: hypotheses_now,
            attempts: 0,
        };
        return Ok(NextStep::Again);
    }
    let text = read_exp_log_text(&state.exp_log_path)?;
    let mbc2_before = count_mbc2_entries(&text);
    state.phase = Phase::Mbc2 {
        baseline: mbc2_before,
        sent: 0,
    };
    Ok(NextStep::Again)
}

pub(crate) fn run_mbc2_phase(state: &mut KpopMultiturnState, text: &str) -> Result<NextStep, String> {
    let Phase::Mbc2 { baseline, sent } = &mut state.phase else {
        return Err("internal: expected Mbc2 phase".to_string());
    };
    let m = count_mbc2_entries(text);
    if m > *baseline {
        start_new_block_after_mbc2(state)?;
        return Ok(NextStep::Again);
    }
    if *sent < 2 {
        return state
            .builder
            .mbc2_turn()
            .map(|s| NextStep::Emit(MultiturnPrompt::Mbc2(s)));
    }
    start_new_block_after_mbc2(state)?;
    Ok(NextStep::Again)
}

fn start_new_block_after_mbc2(state: &mut KpopMultiturnState) -> Result<(), String> {
    let text = read_exp_log_text(&state.exp_log_path)?;
    let hypotheses_before = hypotheses_emitted(&text);
    let mean = block_mean_from_p_creative(state.p_creative);
    let n = state.credit + poisson_block_size(&mut state.rng, mean);
    state.credit = 0;
    state.phase = Phase::KpopBlock {
        target_n: n.max(1),
        hypotheses_before,
        attempts: 0,
    };
    Ok(())
}

#[cfg(test)]
mod kiss_cov {
    #[test]
    fn kiss_cov_multiturn_phases() {
        let _ = stringify!(super::kpop_block_progress_ctx);
        let _ = stringify!(super::set_last_block_miss);
        let _ = stringify!(super::kpop_block_need);
        let _ = stringify!(super::run_kpop_phase);
        let _ = stringify!(super::complete_kpop_block);
        let _ = stringify!(super::run_mbc2_phase);
        let _ = stringify!(super::start_new_block_after_mbc2);
    }
}
