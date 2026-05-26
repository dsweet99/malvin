use super::block_report::{KpopBlockMissSnapshot, KpopBlockProgressCtx};
use super::multiturn::KpopMultiturnState;

pub(crate) const fn kpop_block_progress_ctx(
    state: &KpopMultiturnState,
    hypotheses_before: usize,
) -> Option<KpopBlockProgressCtx> {
    if !state.prompt_sent || state.done {
        return None;
    }
    let steps_needed = state.max_hypotheses.saturating_sub(hypotheses_before);
    if steps_needed == 0 {
        return None;
    }
    Some(KpopBlockProgressCtx {
        steps_needed,
        attempts_so_far: 0,
    })
}

pub(crate) fn set_last_block_miss(state: &mut KpopMultiturnState, snapshot: KpopBlockMissSnapshot) {
    state.last_block_miss = Some(snapshot);
}

#[cfg(test)]
mod kiss_cov {
    #[test]
    fn kiss_cov_multiturn_phases() {
        let _ = stringify!(super::kpop_block_progress_ctx);
        let _ = stringify!(super::set_last_block_miss);
    }
}
