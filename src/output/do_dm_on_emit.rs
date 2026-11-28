use std::sync::Mutex;
use std::time::Duration;

use super::dm_attachment::{completed_direct_messages, existing_attachment};
use super::dm_on_emit_render::{PLACEHOLDER, render_on_emit};
use super::dm_on_emit_run::run_shell_command;
use super::dm_relay::{dm_emit_enabled, in_modal_sandbox, publish_mark};

const ON_EMIT_TIMEOUT: Duration = Duration::from_secs(20);

static COMMAND: Mutex<Option<String>> = Mutex::new(None);
static CONFIG_LOCK: Mutex<()> = Mutex::new(());

pub fn set_dm_on_emit_command(command: Option<String>) {
    let _guard = config_lock();
    store_command(command);
}

fn store_command(command: Option<String>) {
    *lock_command() = command.filter(|value| !value.trim().is_empty());
}

fn config_lock() -> std::sync::MutexGuard<'static, ()> {
    CONFIG_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub(crate) fn notify_completed_direct_messages(text: &str) {
    if !dm_emit_enabled() {
        return;
    }
    if in_modal_sandbox() {
        relay_completed_direct_messages(text);
        return;
    }
    let Some(template) = command_for_emit() else {
        return;
    };
    for message in completed_direct_messages(text) {
        let attachment = existing_attachment(message.attachment);
        if message.body.is_empty() && attachment.is_none() {
            continue;
        }
        deliver(&template, &message.body, attachment.as_deref().unwrap_or(""));
    }
}

fn relay_completed_direct_messages(text: &str) {
    for message in completed_direct_messages(text) {
        if !message.body.is_empty() {
            publish_mark(&message.body);
        }
    }
}

pub(crate) fn deliver_relayed_direct_message(body: &str) {
    if body.is_empty() || !dm_emit_enabled() {
        return;
    }
    if let Some(template) = command_for_emit() {
        deliver(&template, body, "");
    }
}

fn deliver(template: &str, body: &str, attachment: &str) {
    let body = body_with_logspath(body);
    let stdin = (!template.contains(PLACEHOLDER)).then_some(body.as_str());
    run_shell_command(
        &render_on_emit(template, &body, attachment),
        stdin,
        timeout_limit(),
    );
}

fn body_with_logspath(body: &str) -> String {
    let Some(path) = crate::workspace::stdout_log_path::clone_stdout_log_path() else {
        return body.to_string();
    };
    let Some(logs) = path.parent().filter(|dir| !dir.as_os_str().is_empty()) else {
        return body.to_string();
    };
    format!("{}\n{body}", logs.display())
}

fn command_for_emit() -> Option<String> {
    #[cfg(test)]
    {
        if !TEST_ALLOW.with(|flag| *flag.borrow()) {
            return None;
        }
        TEST_COMMAND.with(|slot| slot.borrow().clone())
    }
    #[cfg(not(test))]
    {
        lock_command().clone()
    }
}

fn lock_command() -> std::sync::MutexGuard<'static, Option<String>> {
    COMMAND
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg_attr(not(test), allow(clippy::missing_const_for_fn))]
fn timeout_limit() -> Duration {
    #[cfg(test)]
    if let Some(limit) = TEST_TIMEOUT.with(|slot| *slot.borrow()) {
        return limit;
    }
    ON_EMIT_TIMEOUT
}

#[cfg(test)]
use std::cell::RefCell;

#[cfg(test)]
thread_local! {
    static TEST_ALLOW: RefCell<bool> = const { RefCell::new(false) };
    static TEST_COMMAND: RefCell<Option<String>> = const { RefCell::new(None) };
    static TEST_TIMEOUT: RefCell<Option<Duration>> = const { RefCell::new(None) };
}

#[cfg(test)]
struct TestEmitGuard;

#[cfg(test)]
impl Drop for TestEmitGuard {
    fn drop(&mut self) {
        TEST_TIMEOUT.with(|slot| *slot.borrow_mut() = None);
        TEST_COMMAND.with(|slot| *slot.borrow_mut() = None);
        TEST_ALLOW.with(|flag| *flag.borrow_mut() = false);
    }
}

#[cfg(test)]
pub(crate) fn with_dm_on_emit(command: &str, timeout: Option<Duration>, body: impl FnOnce()) {
    let _guard = TestEmitGuard;
    TEST_ALLOW.with(|flag| *flag.borrow_mut() = true);
    TEST_COMMAND.with(|slot| *slot.borrow_mut() = Some(command.to_string()));
    TEST_TIMEOUT.with(|slot| *slot.borrow_mut() = timeout);
    body();
}

#[cfg(test)]
pub(crate) fn with_config_lock(body: impl FnOnce()) {
    let _guard = config_lock();
    body();
    store_command(None);
}

#[cfg(test)]
pub(crate) fn dm_on_emit_command_for_test() -> Option<String> {
    lock_command().clone()
}

#[cfg(test)]
#[path = "do_dm_on_emit_tests.rs"]
mod do_dm_on_emit_tests;

#[cfg(test)]
#[path = "dm_emit_option_tests.rs"]
mod dm_emit_option_tests;

#[cfg(test)]
#[path = "dm_on_emit_run_tests.rs"]
mod dm_on_emit_run_tests;
