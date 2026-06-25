#[cfg(unix)]
use std::io::Read;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
#[cfg(unix)]
use std::path::Path;
#[cfg(unix)]
use std::process::{Child, Command, Stdio};
#[cfg(unix)]
use std::thread::{self, JoinHandle};
#[cfg(unix)]
use std::time::{Duration, Instant};

#[cfg(unix)]
pub type PipedChildHandles = (Child, JoinHandle<Vec<u8>>, JoinHandle<Vec<u8>>);

#[cfg(unix)]
fn kill_bin() -> &'static Path {
    if Path::new("/bin/kill").is_file() {
        Path::new("/bin/kill")
    } else {
        Path::new("/usr/bin/kill")
    }
}

#[cfg(unix)]
fn kill_process_group(pid: u32) {
    let _ = Command::new(kill_bin())
        .args(["-KILL", &format!("-{pid}")])
        .status();
}

#[cfg(unix)]
fn join_stdio_reader(jh: JoinHandle<Vec<u8>>) -> std::io::Result<Vec<u8>> {
    jh.join()
        .map_err(|_| std::io::Error::other("malvin subprocess reader panicked"))
}

#[cfg(unix)]
fn subprocess_wait_poll_interval() -> Duration {
    if std::env::var_os("MALVIN_TEST_NO_REAL_AGENT").is_some() {
        Duration::from_millis(1)
    } else {
        Duration::from_millis(20)
    }
}

#[cfg(unix)]
fn wait_for_subprocess_tick(spin_budget: &mut u32) {
    if std::env::var_os("MALVIN_TEST_NO_REAL_AGENT").is_some() && *spin_budget > 0 {
        *spin_budget -= 1;
        std::hint::spin_loop();
        return;
    }
    std::thread::sleep(subprocess_wait_poll_interval());
}

#[cfg(unix)]
fn output_joined(
    status: std::process::ExitStatus,
    stdout_jh: JoinHandle<Vec<u8>>,
    stderr_jh: JoinHandle<Vec<u8>>,
) -> std::io::Result<std::process::Output> {
    Ok(std::process::Output {
        status,
        stdout: join_stdio_reader(stdout_jh)?,
        stderr: join_stdio_reader(stderr_jh)?,
    })
}

#[cfg(unix)]
pub fn spawn_piped_process_group(cmd: &mut Command) -> std::io::Result<PipedChildHandles> {
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    cmd.process_group(0);
    let mut child = cmd.spawn()?;
    let stdout = child.stdout.take().expect("stdout piped");
    let stderr = child.stderr.take().expect("stderr piped");
    let stdout_jh = thread::spawn(move || {
        let mut stdout = stdout;
        let mut v = Vec::new();
        let _ = stdout.read_to_end(&mut v);
        v
    });
    let stderr_jh = thread::spawn(move || {
        let mut stderr = stderr;
        let mut v = Vec::new();
        let _ = stderr.read_to_end(&mut v);
        v
    });
    Ok((child, stdout_jh, stderr_jh))
}

#[cfg(unix)]
pub fn wait_child_with_timeout(
    mut child: Child,
    stdout_jh: JoinHandle<Vec<u8>>,
    stderr_jh: JoinHandle<Vec<u8>>,
    deadline: Instant,
) -> std::io::Result<std::process::Output> {
    let mut spin_budget = if std::env::var_os("MALVIN_TEST_NO_REAL_AGENT").is_some() {
        500
    } else {
        0
    };
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return output_joined(status, stdout_jh, stderr_jh),
            Ok(None) => {
                if Instant::now() > deadline {
                    kill_process_group(child.id());
                    let _ = child.wait();
                    let stdout = join_stdio_reader(stdout_jh)?;
                    let stderr = join_stdio_reader(stderr_jh)?;
                    let stdout_text = String::from_utf8_lossy(&stdout);
                    let stderr_text = String::from_utf8_lossy(&stderr);
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        format!(
                            "malvin subprocess timed out; stdout={stdout_text:?}; stderr={stderr_text:?}"
                        ),
                    ));
                }
                wait_for_subprocess_tick(&mut spin_budget);
            }
            Err(e) => {
                let _ = stdout_jh.join();
                let _ = stderr_jh.join();
                return Err(e);
            }
        }
    }
}
