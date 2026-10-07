use std::sync::{Arc, Mutex, OnceLock, Weak};

use super::RunTiming;

fn live_slot() -> &'static Mutex<Option<Weak<Mutex<RunTiming>>>> {
    static LIVE: OnceLock<Mutex<Option<Weak<Mutex<RunTiming>>>>> = OnceLock::new();
    LIVE.get_or_init(|| Mutex::new(None))
}

pub(crate) fn note_live(timing: &Arc<Mutex<RunTiming>>) {
    let mut slot = live_slot()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    *slot = Some(Arc::downgrade(timing));
}

pub(crate) fn flush_open_timing() {
    let Some(arc) = upgraded_live() else {
        return;
    };
    fold_arc(&arc);
}

fn upgraded_live() -> Option<Arc<Mutex<RunTiming>>> {
    live_slot()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
        .and_then(Weak::upgrade)
}

fn fold_arc(timing: &Arc<Mutex<RunTiming>>) {
    let mut guard = timing
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    super::process_ledger::contribute(&mut guard);
}

pub(crate) fn fold_existing(slot: Option<&Arc<Mutex<RunTiming>>>) -> Option<RunTiming> {
    let arc = slot?;
    fold_arc(arc);
    Some(
        arc.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone(),
    )
}

pub(crate) fn replace_tracked_timing(
    slot: &mut Option<Arc<Mutex<RunTiming>>>,
    next: Option<Arc<Mutex<RunTiming>>>,
) {
    if pointer_changed(slot.as_ref(), next.as_ref()) {
        fold_and_clear(slot.as_ref());
    }
    *slot = next;
    if let Some(active) = slot.as_ref() {
        note_live(active);
    }
}

fn pointer_changed(
    slot: Option<&Arc<Mutex<RunTiming>>>,
    next: Option<&Arc<Mutex<RunTiming>>>,
) -> bool {
    match (slot, next) {
        (Some(current), Some(incoming)) => !Arc::ptr_eq(current, incoming),
        (Some(_), None) | (None, Some(_)) => true,
        (None, None) => false,
    }
}

fn fold_and_clear(slot: Option<&Arc<Mutex<RunTiming>>>) {
    let Some(current) = slot else {
        return;
    };
    fold_arc(current);
    clear_live_pointer(current);
}

fn clear_live_pointer(timing: &Arc<Mutex<RunTiming>>) {
    let mut slot = live_slot()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let same = slot
        .as_ref()
        .and_then(Weak::upgrade)
        .is_some_and(|live| Arc::ptr_eq(&live, timing));
    if same {
        *slot = None;
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{flush_open_timing, note_live, replace_tracked_timing};
    use crate::config::malvin_config_file::TokenCostRates;
    use crate::run_timing::RunTiming;

    #[test]
    fn flush_folds_uncommitted_tokens_and_dollars() {
        let timing = RunTiming::new_arc();
        let (tokens_before, cost_before) = {
            let mut guard = timing.lock().expect("lock");
            guard.token_cost_rates = TokenCostRates {
                usd_per_microtoken_in: 1_000.0,
                ..TokenCostRates::default()
            };
            guard.tokens_in = Some(1_000_000);
            (guard.ledger_cursor.tokens_in, guard.ledger_cursor.cost_tot)
        };
        assert_eq!(tokens_before, None);
        assert_eq!(cost_before, None);
        note_live(&timing);
        flush_open_timing();
        let cursor = timing.lock().expect("lock").ledger_cursor.clone();
        assert_eq!(cursor.tokens_in, Some(1_000_000));
        let cost = cursor.cost_tot.expect("cost");
        assert!((cost - 1_000.0).abs() < 1e-9, "cost_tot={cost}");
    }

    #[test]
    fn armed_emit_folds_the_open_turn_before_printing() {
        let _guard = crate::output::STDOUT_LOG_TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        crate::output::set_stdout_log_path(None);
        crate::output::enable_stdout_capture();
        let timing = RunTiming::new_arc();
        timing.lock().expect("lock").tokens_out = Some(4);
        note_live(&timing);
        crate::run_timing::arm_process_footnotes();
        crate::run_timing::emit_process_footnotes_if_armed();
        let _ = crate::output::take_captured_stdout();
        assert_eq!(
            timing.lock().expect("lock").ledger_cursor.tokens_out,
            Some(4)
        );
    }

    #[test]
    fn clearing_the_slot_folds_before_the_handle_is_dropped() {
        let timing = RunTiming::new_arc();
        timing.lock().expect("lock").steps = 3;
        let mut slot = Some(Arc::clone(&timing));
        note_live(&timing);
        replace_tracked_timing(&mut slot, None);
        assert!(slot.is_none());
        assert_eq!(timing.lock().expect("lock").ledger_cursor.steps, 3);
        flush_open_timing();
        assert_eq!(timing.lock().expect("lock").ledger_cursor.steps, 3);
    }
}
