#[path = "dm_relay_mark.rs"]
mod dm_relay_mark;

use std::sync::atomic::{AtomicBool, Ordering};

static EMIT_ENABLED: AtomicBool = AtomicBool::new(true);

pub fn set_dm_emit_enabled(enabled: bool) {
    EMIT_ENABLED.store(enabled, Ordering::Relaxed);
}

pub(super) fn dm_emit_enabled() -> bool {
    EMIT_ENABLED.load(Ordering::Relaxed)
}

#[cfg(test)]
pub(crate) use dm_relay_mark::mark_line;
pub(super) use dm_relay_mark::publish_mark;
pub(crate) use dm_relay_mark::parse_dm_emit_mark;

#[cfg(test)]
thread_local! {
    static FORCE_MODAL: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub(super) fn in_modal_sandbox() -> bool {
    #[cfg(test)]
    if FORCE_MODAL.with(std::cell::Cell::get) {
        return true;
    }
    crate::env_flag::env_flag("MALVIN_MODAL_REMOTE")
}

#[cfg(test)]
pub(crate) fn force_modal_mark(on: bool) {
    FORCE_MODAL.with(|flag| flag.set(on));
}

#[cfg(test)]
pub(crate) fn take_test_marks() -> Vec<String> {
    dm_relay_mark::take_test_marks()
}

#[cfg(test)]
pub(crate) fn clear_test_marks() {
    dm_relay_mark::clear_test_marks();
}
