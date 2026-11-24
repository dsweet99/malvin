use std::ffi::{OsStr, OsString};
use std::process::Stdio;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Worker {
    Summary,
    FactAgent,
    HerdrLinger,
    LocalLlmManager,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Lifetime {
    Attached,
    ProcessGroup,
    Daemon,
}

impl Worker {
    const ALL: [Self; 4] = [
        Self::Summary,
        Self::FactAgent,
        Self::HerdrLinger,
        Self::LocalLlmManager,
    ];

    #[must_use]
    pub const fn marker(self) -> &'static str {
        match self {
            Self::Summary => "__summary-worker",
            Self::FactAgent => "__fact-agent-worker",
            Self::HerdrLinger => "__herdr-linger",
            Self::LocalLlmManager => "__local-llm-manager",
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Summary => "request summary worker",
            Self::FactAgent => "fact agent",
            Self::HerdrLinger => "herdr linger",
            Self::LocalLlmManager => "local llm manager",
        }
    }

    const fn lifetime(self) -> Lifetime {
        match self {
            Self::Summary | Self::FactAgent => Lifetime::Attached,
            Self::HerdrLinger => Lifetime::ProcessGroup,
            Self::LocalLlmManager => Lifetime::Daemon,
        }
    }
}

#[must_use]
pub fn invocation(mut args: impl Iterator<Item = OsString>) -> Option<(Worker, Vec<OsString>)> {
    let first = args.next()?;
    let worker = Worker::ALL.into_iter().find(|w| first == w.marker())?;
    Some((worker, args.collect()))
}

#[must_use]
pub fn run_if_worker() -> Option<crate::Exit> {
    let (worker, args) = invocation(std::env::args_os().skip(1))?;
    let result = match worker {
        Worker::Summary => crate::task_summary::run_worker(args),
        Worker::FactAgent => crate::fact_worker::run_worker(args),
        Worker::HerdrLinger => crate::herdr::run_linger_worker(args),
        Worker::LocalLlmManager => crate::local_llm::run_local_llm_manager(),
    };
    Some(match result {
        Ok(()) => crate::Exit::Success,
        Err(error) => {
            eprintln!("{}: {error}", worker.label());
            crate::Exit::Failure
        }
    })
}

#[must_use]
pub(crate) fn is_worker_cmdline(cmdline: &[u8]) -> bool {
    cmdline
        .split(|&b| b == 0)
        .nth(1)
        .is_some_and(|arg| Worker::ALL.iter().any(|w| arg == w.marker().as_bytes()))
}

pub(crate) fn usage(worker: Worker, operands: &str) -> String {
    format!("usage: malvin {} {operands}", worker.marker())
}

fn base_command<I, S>(worker: Worker, args: I) -> Result<std::process::Command, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let exe = crate::self_exe::self_exe()
        .map_err(|e| format!("cannot locate the malvin executable: {e}"))?;
    let mut command = std::process::Command::new(exe);
    command
        .arg(worker.marker())
        .args(args)
        .env("MALVIN_NO_HERDR", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    crate::herdr::strip_herdr_env(&mut command);
    Ok(command)
}

pub(crate) fn attached_command<I, S>(
    worker: Worker,
    args: I,
    stderr: std::fs::File,
) -> Result<tokio::process::Command, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    if worker.lifetime() != Lifetime::Attached {
        return Err(format!("{} is not an attached worker", worker.marker()));
    }
    let mut command = base_command(worker, args)?;
    command.stdin(Stdio::piped()).stderr(stderr);
    let mut command = tokio::process::Command::from(command);
    command.kill_on_drop(true);
    Ok(command)
}

pub(crate) fn spawn_detached<I, S>(worker: Worker, args: I) -> Result<u32, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut child = detached_command(worker, args)?
        .spawn()
        .map_err(|e| format!("spawn {}: {e}", worker.label()))?;
    let pid = child.id();
    match worker.lifetime() {
        Lifetime::Daemon => {
            child
                .wait()
                .map_err(|e| format!("wait for {} launcher: {e}", worker.label()))?;
        }
        Lifetime::Attached | Lifetime::ProcessGroup => {
            std::thread::spawn(move || child.wait());
        }
    }
    Ok(pid)
}

fn detached_command<I, S>(worker: Worker, args: I) -> Result<std::process::Command, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = base_command(worker, args)?;
    match worker.lifetime() {
        Lifetime::Attached => {
            return Err(format!("{} is not a detached worker", worker.marker()));
        }
        Lifetime::ProcessGroup => {
            std::os::unix::process::CommandExt::process_group(&mut command, 0);
        }
        Lifetime::Daemon => daemonize(&mut command),
    }
    Ok(command)
}

#[allow(unsafe_code)]
fn daemonize(command: &mut std::process::Command) {
    use std::os::unix::process::CommandExt;
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() == -1 {
                return Err(std::io::Error::last_os_error());
            }
            match libc::fork() {
                -1 => Err(std::io::Error::last_os_error()),
                0 => Ok(()),
                _ => libc::_exit(0),
            }
        });
    }
}

pub(crate) fn parent_disconnected() -> tokio::sync::oneshot::Receiver<()> {
    let (closed, disconnected) = tokio::sync::oneshot::channel();
    std::thread::spawn(move || {
        let _ = std::io::Read::read_to_end(&mut std::io::stdin(), &mut Vec::new());
        let _ = closed.send(());
    });
    disconnected
}

#[cfg(test)]
#[path = "self_worker_tests.rs"]
mod tests;
