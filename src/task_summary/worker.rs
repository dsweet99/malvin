use std::io::Read;
use std::path::Path;
use std::process::Stdio;
use std::time::{Duration, Instant};

use crate::artifacts::RunArtifacts;
use crate::config::model_list::ModelList;

const PROMPT_POLL: Duration = Duration::from_millis(50);

struct StatusClock {
    due: Instant,
    period: Duration,
    seen_prompts: u64,
    timer_due: bool,
}

const WORKER_REQUEST: &str = "MALVIN_SUMMARY_WORKER_REQUEST";

pub(super) fn spawn(artifacts: &RunArtifacts) -> Result<tokio::process::Child, String> {
    let mut command = worker_command(artifacts)?;
    command.kill_on_drop(true);
    command.spawn().map_err(|e| e.to_string())
}

fn worker_command(artifacts: &RunArtifacts) -> Result<tokio::process::Command, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let log = std::fs::File::create(artifacts.run_dir.join("request_summary.log"))
        .map_err(|e| e.to_string())?;
    let mut command = tokio::process::Command::new(exe);
    command
        .current_dir(&artifacts.work_dir)
        .env(WORKER_REQUEST, &artifacts.plan_path)
        .env("MALVIN_NO_HERDR", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(log);
    crate::herdr::strip_herdr_env_tokio(&mut command);
    Ok(command)
}

pub fn run_if_worker() -> Option<crate::Exit> {
    let request = std::env::var_os(WORKER_REQUEST)?;
    let result = run(std::path::PathBuf::from(request));
    Some(match result {
        Ok(()) => crate::Exit::Success,
        Err(error) => {
            eprintln!("{error}");
            crate::Exit::Failure
        }
    })
}

fn run(plan_path: std::path::PathBuf) -> Result<(), String> {
    let work_dir = std::env::current_dir().map_err(|e| e.to_string())?;
    let run_dir = plan_path
        .parent()
        .ok_or("request has no parent directory")?
        .to_path_buf();
    let config = crate::config::malvin_config_file::load_malvin_config(&work_dir);
    let models = config.agent.model_lite;
    let artifacts = RunArtifacts {
        run_dir,
        plan_path,
        work_dir,
    };
    let (closed, disconnected) = tokio::sync::oneshot::channel();
    std::thread::spawn(move || {
        let _ = std::io::stdin().read_to_end(&mut Vec::new());
        let _ = closed.send(());
    });
    let runtime = worker_runtime()?;
    runtime.block_on(session(&artifacts, models.as_ref(), disconnected))
}

async fn session(
    artifacts: &RunArtifacts,
    models: Option<&ModelList>,
    mut disconnected: tokio::sync::oneshot::Receiver<()>,
) -> Result<(), String> {
    let started = std::time::Instant::now();
    let task = tokio::select! {
        _ = &mut disconnected => return Ok(()),
        result = super::generate_task(artifacts, models) => result,
    };
    task?;
    if let Some(models) = models {
        status_loop(artifacts, models, started, &mut disconnected).await?;
    }
    Ok(())
}

fn status_clock(started: Instant) -> StatusClock {
    let (first, period) = super::schedule::status_delays();
    StatusClock {
        due: started + first,
        period,
        seen_prompts: 0,
        timer_due: false,
    }
}

fn prompt_send_count(run_dir: &Path) -> u64 {
    let path = run_dir.join(crate::malvin_constants::PROMPT_SENDS);
    std::fs::metadata(path).map_or(0, |meta| meta.len())
}

async fn status_loop(
    artifacts: &RunArtifacts,
    models: &ModelList,
    started: Instant,
    disconnected: &mut tokio::sync::oneshot::Receiver<()>,
) -> Result<(), String> {
    let mut clock = status_clock(started);
    loop {
        if wait_until_status_due(artifacts, &mut clock, disconnected).await? {
            return Ok(());
        }
        if run_status_once(artifacts, models, disconnected).await? {
            return Ok(());
        }
        if clock.timer_due {
            clock.due = super::schedule::next_due(clock.due, clock.period, Instant::now());
        }
    }
}

async fn wait_until_status_due(
    artifacts: &RunArtifacts,
    clock: &mut StatusClock,
    disconnected: &mut tokio::sync::oneshot::Receiver<()>,
) -> Result<bool, String> {
    loop {
        let now = Instant::now();
        let wake = super::schedule::status_wake(super::schedule::StatusSample {
            now,
            due: clock.due,
            seen_prompts: clock.seen_prompts,
            prompt_count: prompt_send_count(&artifacts.run_dir),
        });
        clock.seen_prompts = wake.seen_prompts;
        clock.timer_due = wake.timer_due;
        if wake.generate {
            return Ok(false);
        }
        let wait = clock.due.saturating_duration_since(now).min(PROMPT_POLL);
        tokio::select! {
            _ = &mut *disconnected => return Ok(true),
            () = tokio::time::sleep(wait) => {}
        }
    }
}

async fn run_status_once(
    artifacts: &RunArtifacts,
    models: &ModelList,
    disconnected: &mut tokio::sync::oneshot::Receiver<()>,
) -> Result<bool, String> {
    let result = tokio::select! {
        _ = &mut *disconnected => return Ok(true),
        result = super::generate_status(artifacts, models) => result,
    };
    if let Err(error) = result {
        eprintln!("{error}");
    }
    Ok(false)
}

fn worker_runtime() -> Result<tokio::runtime::Runtime, String> {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .map_err(|e| e.to_string())
}
