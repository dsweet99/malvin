use std::io::Read;
use std::process::Stdio;

use crate::artifacts::RunArtifacts;
use crate::config::model_list::ModelList;

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

async fn status_loop(
    artifacts: &RunArtifacts,
    models: &ModelList,
    started: std::time::Instant,
    disconnected: &mut tokio::sync::oneshot::Receiver<()>,
) -> Result<(), String> {
    let (first, period) = super::schedule::status_delays();
    let mut due = started + first;
    loop {
        let wait = due.saturating_duration_since(std::time::Instant::now());
        tokio::select! {
            _ = &mut *disconnected => return Ok(()),
            () = tokio::time::sleep(wait) => {}
        }
        let result = tokio::select! {
            _ = &mut *disconnected => return Ok(()),
            result = super::generate_status(artifacts, models) => result,
        };
        if let Err(error) = result {
            eprintln!("{error}");
        }
        due = super::schedule::next_due(due, period, std::time::Instant::now());
    }
}

fn worker_runtime() -> Result<tokio::runtime::Runtime, String> {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .map_err(|e| e.to_string())
}
