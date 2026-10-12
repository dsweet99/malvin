use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use super::next_dm_index;

pub(crate) fn exec_hook(argv: &[String], stdin_body: Option<&str>, timeout: Duration) {
    let Some(name) = argv.first().cloned() else {
        return;
    };
    let index = next_dm_index();
    match spawn_hook(argv, stdin_body.is_some(), index) {
        Ok(child) => finish_child(child, &name, stdin_body.map(str::to_string), timeout),
        Err(err) => report(&format!("{name}: {err}"), "", ""),
    }
}

fn spawn_hook(argv: &[String], pipe_stdin: bool, index: u64) -> Result<Child, String> {
    let mut cmd = Command::new(&argv[0]);
    cmd.args(&argv[1..]);
    apply_hook_env(&mut cmd, index);
    cmd.stdin(if pipe_stdin {
        Stdio::piped()
    } else {
        Stdio::null()
    });
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    cmd.spawn().map_err(|err| format!("{err}"))
}

fn apply_hook_env(cmd: &mut Command, index: u64) {
    let work = std::env::current_dir()
        .map(|path| path.display().to_string())
        .unwrap_or_default();
    let run = crate::workspace::run_id::active_run_dir()
        .map(|path| path.display().to_string())
        .unwrap_or_default();
    cmd.env("MALVIN_DM_INDEX", index.to_string());
    cmd.env("MALVIN_RUN_DIR", run);
    cmd.env("MALVIN_WORKSPACE", &work);
    if !work.is_empty() {
        cmd.current_dir(work);
    }
}

fn finish_child(mut child: Child, name: &str, stdin_body: Option<String>, timeout: Duration) {
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let stdin = child.stdin.take();
    let out_thread = stdout.map(|pipe| thread::spawn(move || read_pipe(pipe)));
    let err_thread = stderr.map(|pipe| thread::spawn(move || read_pipe(pipe)));
    let in_thread = thread::spawn(move || write_pipe(stdin, stdin_body));
    let label = wait_label(&mut child, timeout);
    let _ = in_thread.join();
    let out = join_text(out_thread);
    let err = join_text(err_thread);
    report(&format!("{name} {label}"), &out, &err);
}

fn read_pipe(mut pipe: impl Read) -> String {
    let mut buf = String::new();
    let _ = pipe.read_to_string(&mut buf);
    buf
}

fn write_pipe(pipe: Option<impl Write>, body: Option<String>) {
    let (Some(mut pipe), Some(body)) = (pipe, body) else {
        return;
    };
    let _ = pipe.write_all(body.as_bytes());
}

fn join_text(handle: Option<thread::JoinHandle<String>>) -> String {
    handle
        .and_then(|handle| handle.join().ok())
        .unwrap_or_default()
}

fn wait_label(child: &mut Child, timeout: Duration) -> String {
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return exit_label(status),
            Ok(None) if start.elapsed() >= timeout => return kill_child(child),
            Ok(None) => thread::sleep(Duration::from_millis(20)),
            Err(err) => return format!("wait failed: {err}"),
        }
    }
}

fn exit_label(status: std::process::ExitStatus) -> String {
    status.code().map_or_else(
        || "exited without a status".to_string(),
        |code| format!("exited {code}"),
    )
}

fn kill_child(child: &mut Child) -> String {
    kill_group(child.id());
    let _ = child.kill();
    let _ = child.wait();
    "timed out".to_string()
}

fn kill_group(pid: u32) {
    let _ = Command::new("kill")
        .args(["-KILL", &format!("-{pid}")])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

fn report(summary: &str, stdout: &str, stderr: &str) {
    append_hook_log(summary, stdout, stderr);
    if !summary.ends_with(" exited 0") {
        super::super::print_log_warning(&format!("dm hook: {summary}"));
    }
}

fn append_hook_log(summary: &str, stdout: &str, stderr: &str) {
    let mut text = format!("dm-hook: {summary}\n");
    push_labeled(&mut text, "stdout", stdout);
    push_labeled(&mut text, "stderr", stderr);
    for line in text.lines() {
        super::super::append_stdout_log_line(&format!("dm-hook|{line}"));
    }
}

fn push_labeled(text: &mut String, label: &str, block: &str) {
    if block.is_empty() {
        return;
    }
    text.push_str(label);
    text.push_str(":\n");
    push_block(text, block);
}

fn push_block(text: &mut String, block: &str) {
    if block.is_empty() {
        return;
    }
    text.push_str(block);
    if !block.ends_with('\n') {
        text.push('\n');
    }
}
