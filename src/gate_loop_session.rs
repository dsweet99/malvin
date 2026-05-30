//! Active gate-loop iteration slot for malvin-owned session artifacts (OOM markers, etc.).

use std::sync::{Mutex, PoisonError};

static ACTIVE_GATE_ITERATION: Mutex<Option<usize>> = Mutex::new(None);

/// Records the gate-loop iteration for the in-flight ACP session, if any.
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
}
