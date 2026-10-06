use std::sync::{Arc, MutexGuard, PoisonError};
use std::time::Instant;

use crate::agent_process::TraceChunkEmission;

#[cfg(not(test))]
use super::write_coalesced_emissions;
use super::{CoalesceSlot, Shared, lock_slot};

pub(super) fn worker_loop(shared: Arc<Shared>) {
    let mut slot = lock_slot(&shared);
    while !slot.stop {
        slot = wait_for_deadline(&shared, slot);
        if slot.stop || !deadline_due(&slot) {
            continue;
        }
        let emissions = take_due(&mut slot);
        drop(slot);
        deliver(&shared, emissions);
        slot = lock_slot(&shared);
    }
    drop(slot);
}

fn wait_for_deadline<'a>(
    shared: &'a Shared,
    mut slot: MutexGuard<'a, CoalesceSlot>,
) -> MutexGuard<'a, CoalesceSlot> {
    loop {
        if slot.stop {
            return slot;
        }
        let Some(deadline) = slot.deadline else {
            slot = wait_cv(shared, slot);
            continue;
        };
        if Instant::now() >= deadline {
            return slot;
        }
        slot = wait_timeout(shared, slot, deadline);
    }
}

fn wait_cv<'a>(
    shared: &'a Shared,
    slot: MutexGuard<'a, CoalesceSlot>,
) -> MutexGuard<'a, CoalesceSlot> {
    shared.cv.wait(slot).unwrap_or_else(PoisonError::into_inner)
}

fn wait_timeout<'a>(
    shared: &'a Shared,
    slot: MutexGuard<'a, CoalesceSlot>,
    deadline: Instant,
) -> MutexGuard<'a, CoalesceSlot> {
    let remaining = deadline.saturating_duration_since(Instant::now());
    let (guard, _) = shared
        .cv
        .wait_timeout(slot, remaining)
        .unwrap_or_else(PoisonError::into_inner);
    guard
}

fn deadline_due(slot: &CoalesceSlot) -> bool {
    slot.deadline
        .is_some_and(|deadline| Instant::now() >= deadline)
}

fn take_due(slot: &mut CoalesceSlot) -> Vec<TraceChunkEmission> {
    slot.deadline = None;
    slot.inner.flush_all()
}

fn deliver(shared: &Shared, emissions: Vec<TraceChunkEmission>) {
    #[cfg(test)]
    remember(shared, &emissions);
    #[cfg(not(test))]
    write_coalesced_emissions(shared.io, emissions);
}

#[cfg(test)]
fn remember(shared: &Shared, emissions: &[TraceChunkEmission]) {
    let mut lines = shared
        .test_lines
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    for (_, line, ..) in emissions {
        lines.push(line.clone());
    }
}
