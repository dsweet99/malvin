use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

static ACTIVE_GATE_ITERATION: Mutex<Option<usize>> = Mutex::new(None);
static QUALITY_GATES_JUST_RAN: Mutex<bool> = Mutex::new(false);
static ROUTER_SESSION_START: Mutex<Option<Instant>> = Mutex::new(None);

fn set_router_session_start(at: Option<Instant>) {
    *ROUTER_SESSION_START
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = at;
}

#[must_use]
pub fn router_session_elapsed() -> Option<Duration> {
    let start = *ROUTER_SESSION_START
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    elapsed_between(start, Instant::now())
}

fn elapsed_between(start: Option<Instant>, now: Instant) -> Option<Duration> {
    start.map(|s| now.saturating_duration_since(s))
}

pub fn set_active_gate_iteration(iteration: Option<usize>) {
    *ACTIVE_GATE_ITERATION
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = iteration;
}

#[must_use]
pub fn active_gate_iteration() -> Option<usize> {
    *ACTIVE_GATE_ITERATION
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

pub fn set_quality_gates_just_ran(ran: bool) {
    *QUALITY_GATES_JUST_RAN
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = ran;
}

#[must_use]
pub fn quality_gates_just_ran() -> bool {
    *QUALITY_GATES_JUST_RAN
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

pub fn reset_for_independent_run() {
    set_active_gate_iteration(None);
    set_quality_gates_just_ran(false);
    set_router_session_start(Some(Instant::now()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_gate_iteration_round_trip() {
        set_active_gate_iteration(Some(3));
        assert_eq!(active_gate_iteration(), Some(3));
        set_active_gate_iteration(None);
        assert_eq!(active_gate_iteration(), None);
    }

    #[test]
    fn quality_gates_just_ran_round_trip() {
        set_quality_gates_just_ran(true);
        assert!(quality_gates_just_ran());
        set_quality_gates_just_ran(false);
        assert!(!quality_gates_just_ran());
    }

    #[test]
    fn reset_for_independent_run_clears_process_global_gate_flags() {
        set_active_gate_iteration(Some(9));
        set_quality_gates_just_ran(true);
        reset_for_independent_run();
        assert_eq!(active_gate_iteration(), None);
        assert!(!quality_gates_just_ran());
        assert!(router_session_elapsed().is_some_and(|d| d < Duration::from_mins(1)));
    }

    #[test]
    fn elapsed_between_measures_from_recorded_start() {
        let now = Instant::now();
        let start = now.checked_sub(Duration::from_secs(90)).expect("instant");
        assert_eq!(
            elapsed_between(Some(start), now),
            Some(Duration::from_secs(90))
        );
        assert_eq!(elapsed_between(None, now), None);
    }
}
