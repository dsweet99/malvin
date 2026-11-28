use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use super::dm_stderr::stderr_snippet;

#[cfg(not(test))]
static DM_INDEX: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

#[cfg(test)]
thread_local! {
    static TEST_INDEX: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

fn next_dm_index() -> u64 {
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

#[cfg(test)]
pub(super) fn reset_dm_index_for_test() {
    TEST_INDEX.with(|index| index.set(0));
}

pub(super) fn run_shell_command(command: &str, stdin_body: Option<&str>, timeout: Duration) {
    let mut cmd = Command::new("sh");
    cmd.arg("-c")
        .arg(command)
        .stdin(if stdin_body.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    apply_hook_env(&mut cmd, next_dm_index());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    match cmd.spawn() {
        Ok(child) => finish(child, stdin_body.map(str::to_string), timeout),
        Err(err) => super::print_log_warning(&format!("dm.on_emit failed to start: {err}")),
    }
}

fn apply_hook_env(cmd: &mut Command, index: u64) {
    let work = std::env::current_dir()
        .map(|path| path.display().to_string())
        .unwrap_or_default();
    let run = crate::workspace::run_id::active_run_dir()
        .map(|path| path.display().to_string())
        .unwrap_or_default();
    cmd.env("MALVIN_DM_INDEX", index.to_string())
        .env("MALVIN_RUN_DIR", run)
        .env("MALVIN_WORKSPACE", work);
}

fn finish(mut child: Child, stdin_body: Option<String>, timeout: Duration) {
    let writer = child.stdin.take().map(|mut pipe| {
        std::thread::spawn(move || {
            let _ = pipe.write_all(stdin_body.unwrap_or_default().as_bytes());
        })
    });
    let waited = crate::command_output_timeout::wait_child_with_timeout(
        &mut child,
        timeout,
        "dm.on_emit",
    );
    if let Some(writer) = writer {
        let _ = writer.join();
    }
    match waited {
        Ok(status) if status.success() => {}
        Ok(status) => {
            let detail = stderr_snippet(&mut child);
            super::print_log_warning(&format!("dm.on_emit exited {status}{detail}"));
        }
        Err(message) => super::print_log_warning(&message),
    }
}
