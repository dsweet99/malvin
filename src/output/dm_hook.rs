#[path = "dm_hook_config.rs"]
mod dm_hook_config;
#[path = "dm_hook_exec.rs"]
mod dm_hook_exec;
#[path = "dm_hook_mark.rs"]
mod dm_hook_mark;
#[path = "dm_hook_split.rs"]
mod dm_hook_split;

use std::cell::Cell;
use std::time::Duration;

#[cfg(test)]
pub(crate) use dm_hook_mark::mark_line;
pub(crate) use dm_hook_mark::parse_dm_emit_mark;

const DM_HOOK_TIMEOUT: Duration = Duration::from_secs(10);

#[cfg(not(test))]
static DM_INDEX: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

thread_local! {
    static TEST_HOOK: std::cell::RefCell<Option<Vec<String>>> =
        const { std::cell::RefCell::new(None) };
    static FORCE_MODAL: Cell<bool> = const { Cell::new(false) };
    static TEST_INDEX: Cell<u64> = const { Cell::new(0) };
}

pub(crate) fn deliver_finished_dm(body: &str) {
    if body.is_empty() {
        return;
    }
    if in_modal_sandbox() {
        dm_hook_mark::publish_mark(body);
        return;
    }
    let Some(tokens) = configured_tokens() else {
        return;
    };
    let (argv, via_arg) = dm_hook_split::bind_body(&tokens, body);
    if argv.is_empty() {
        return;
    }
    let stdin = (!via_arg).then_some(body);
    dm_hook_exec::exec_hook(&argv, stdin, DM_HOOK_TIMEOUT);
}

pub(crate) fn next_dm_index() -> u64 {
    #[cfg(test)]
    {
        TEST_INDEX.with(|index| {
            let next = index.get() + 1;
            index.set(next);
            next
        })
    }
    #[cfg(not(test))]
    {
        DM_INDEX.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1
    }
}

fn configured_tokens() -> Option<Vec<String>> {
    #[cfg(test)]
    {
        TEST_HOOK.with(|hook| hook.borrow().clone())
    }
    #[cfg(not(test))]
    {
        dm_hook_config::load_home_tokens()
    }
}

fn in_modal_sandbox() -> bool {
    if FORCE_MODAL.with(Cell::get) {
        return true;
    }
    std::env::var("MALVIN_MODAL_REMOTE").ok().as_deref() == Some("1")
}

#[cfg(test)]
pub(crate) fn set_test_dm_hook(cmd: Option<Vec<String>>) {
    TEST_HOOK.with(|hook| *hook.borrow_mut() = cmd);
}

#[cfg(test)]
pub(crate) fn force_modal_mark(on: bool) {
    FORCE_MODAL.with(|flag| flag.set(on));
}

#[cfg(test)]
pub(crate) fn reset_dm_hook_state() {
    TEST_INDEX.with(|index| index.set(0));
    force_modal_mark(false);
    set_test_dm_hook(None);
    dm_hook_mark::clear_test_marks();
}

#[cfg(test)]
pub(crate) fn take_test_marks() -> Vec<String> {
    dm_hook_mark::take_test_marks()
}

#[cfg(test)]
#[path = "dm_hook_tests.rs"]
mod tests;
