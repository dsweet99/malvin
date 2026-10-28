use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use super::dm_attachment::completed_direct_messages;
use super::dm_stderr::stderr_snippet;

const PLACEHOLDER: &str = "{body}";
const ATTACHMENT_PLACEHOLDER: &str = "{attachment}";
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
    if !super::dm_hook::dm_emit_enabled() {
        return;
    }
    let Some(template) = command_for_emit() else {
        return;
    };
    for message in completed_direct_messages(text) {
        if message.body.is_empty() && message.attachment.is_none() {
            continue;
        }
        let body = body_with_logspath(&message.body);
        run_shell_command(&render_on_emit(
            &template,
            &body,
            message.attachment.as_deref().unwrap_or(""),
        ));
    }
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

fn render_on_emit(template: &str, body: &str, attachment: &str) -> String {
    let mut rendered = String::new();
    let mut rest = template;
    let mut quote = None;
    while let Some((index, placeholder, value)) = next_placeholder(rest, body, attachment) {
        let (start, stop) = placeholder_span(rest, index, placeholder);
        rendered.push_str(&rest[..start]);
        quote = quote_after(&rest[..start], quote);
        rendered.push_str(&render_placeholder(value, quote));
        rest = &rest[stop..];
    }
    rendered.push_str(rest);
    rendered
}

fn quote_after(text: &str, mut quote: Option<char>) -> Option<char> {
    for ch in text.chars() {
        if quote == Some(ch) {
            quote = None;
        } else if quote.is_none() && matches!(ch, '\'' | '"') {
            quote = Some(ch);
        }
    }
    quote
}

fn render_placeholder(value: &str, quote: Option<char>) -> String {
    match quote {
        Some('\'') => value.replace('\'', "'\\''"),
        Some('"') => double_quote_content(value),
        None => shell_single_quote(value),
        _ => shell_single_quote(value),
    }
}

fn double_quote_content(value: &str) -> String {
    let mut content = String::new();
    for ch in value.chars() {
        if matches!(ch, '\\' | '$' | '`' | '"') {
            content.push('\\');
        }
        content.push(ch);
    }
    content
}

fn next_placeholder<'a>(
    template: &'a str,
    body: &'a str,
    attachment: &'a str,
) -> Option<(usize, &'static str, &'a str)> {
    match (
        template.find(PLACEHOLDER),
        template.find(ATTACHMENT_PLACEHOLDER),
    ) {
        (Some(body_index), Some(attachment_index)) if body_index <= attachment_index => {
            Some((body_index, PLACEHOLDER, body))
        }
        (Some(_), Some(attachment_index)) => {
            Some((attachment_index, ATTACHMENT_PLACEHOLDER, attachment))
        }
        (Some(body_index), None) => Some((body_index, PLACEHOLDER, body)),
        (None, Some(attachment_index)) => {
            Some((attachment_index, ATTACHMENT_PLACEHOLDER, attachment))
        }
        (None, None) => None,
    }
}

fn placeholder_span(rest: &str, index: usize, placeholder: &str) -> (usize, usize) {
    let end = index + placeholder.len();
    if index > 0 {
        let open = rest.as_bytes()[index - 1];
        let close = rest.as_bytes().get(end).copied();
        if (open == b'\'' || open == b'"') && close == Some(open) {
            return (index - 1, end + 1);
        }
    }
    (index, end)
}

fn shell_single_quote(text: &str) -> String {
    let mut out = String::from("'");
    for ch in text.chars() {
        if ch == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(ch);
        }
    }
    out.push('\'');
    out
}

fn run_shell_command(command: &str) {
    let mut child = match Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(err) => {
            super::print_log_warning(&format!("dm.on_emit failed to start: {err}"));
            return;
        }
    };
    wait_bounded(&mut child);
}

fn wait_bounded(child: &mut Child) {
    let started = Instant::now();
    let limit = timeout_limit();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                warn_if_failed(child, status);
                return;
            }
            Ok(None) if started.elapsed() >= limit => {
                let _ = child.kill();
                let _ = child.wait();
                super::print_log_warning("dm.on_emit timed out");
                return;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            Err(err) => {
                super::print_log_warning(&format!("dm.on_emit wait failed: {err}"));
                return;
            }
        }
    }
}

#[cfg_attr(not(test), allow(clippy::missing_const_for_fn))]
fn timeout_limit() -> Duration {
    #[cfg(test)]
    if let Some(limit) = TEST_TIMEOUT.with(|slot| *slot.borrow()) {
        return limit;
    }
    ON_EMIT_TIMEOUT
}

fn warn_if_failed(child: &mut Child, status: ExitStatus) {
    if status.success() {
        return;
    }
    let detail = stderr_snippet(child);
    super::print_log_warning(&format!("dm.on_emit exited {status}{detail}"));
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
