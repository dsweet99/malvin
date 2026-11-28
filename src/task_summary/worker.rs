use std::time::Instant;

use crate::artifacts::RunArtifacts;
use crate::config::model_list::ModelList;

use crate::self_worker::Worker;

pub(super) fn spawn(artifacts: &RunArtifacts) -> Result<tokio::process::Child, String> {
    worker_command(artifacts)?
        .spawn()
        .map_err(|e| e.to_string())
}

pub(crate) fn worker_command(artifacts: &RunArtifacts) -> Result<tokio::process::Command, String> {
    let log = std::fs::File::create(artifacts.run_dir.join("request_summary.log"))
        .map_err(|e| e.to_string())?;
    let mut command =
        crate::self_worker::attached_command(Worker::Summary, [&artifacts.plan_path], log)?;
    command.current_dir(&artifacts.work_dir);
    Ok(command)
}

pub fn run_worker(args: Vec<std::ffi::OsString>) -> Result<(), String> {
    run(worker_request(args)?)
}

pub(crate) fn worker_request(args: Vec<std::ffi::OsString>) -> Result<std::path::PathBuf, String> {
    match <[std::ffi::OsString; 1]>::try_from(args) {
        Ok([plan_path]) => Ok(std::path::PathBuf::from(plan_path)),
        Err(_) => Err(crate::self_worker::usage(Worker::Summary, "REQUEST_PATH")),
    }
}

fn run(plan_path: std::path::PathBuf) -> Result<(), String> {
    let work_dir = std::env::current_dir().map_err(|e| e.to_string())?;
    let run_dir = plan_path
        .parent()
        .ok_or("request has no parent directory")?
        .to_path_buf();
    let config = crate::config::malvin_config_file::load_malvin_config();
    let models = config.agent.model_lite;
    let artifacts = RunArtifacts {
        run_dir,
        plan_path,
        work_dir,
    };
    let disconnected = crate::self_worker::parent_disconnected();
    let runtime = worker_runtime()?;
    runtime.block_on(session(&artifacts, models.as_ref(), disconnected))
}

async fn session(
    artifacts: &RunArtifacts,
    models: Option<&ModelList>,
    mut disconnected: tokio::sync::oneshot::Receiver<()>,
) -> Result<(), String> {
    let origin = Instant::now();
    let task = tokio::select! {
        _ = &mut disconnected => return Ok(()),
        result = super::generate_task(artifacts, models) => result,
    };
    task?;
    if let Some(models) = models {
        status_loop(artifacts, models, origin, &mut disconnected).await?;
    }
    Ok(())
}

async fn status_loop(
    artifacts: &RunArtifacts,
    models: &ModelList,
    origin: Instant,
    disconnected: &mut tokio::sync::oneshot::Receiver<()>,
) -> Result<(), String> {
    for index in 0.. {
        let due = origin + super::schedule::status_offset(index);
        tokio::select! {
            _ = &mut *disconnected => return Ok(()),
            () = tokio::time::sleep_until(due.into()) => {}
        }
        if run_status_once(artifacts, models, disconnected).await? {
            return Ok(());
        }
    }
    Ok(())
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

pub(crate) fn worker_runtime() -> Result<tokio::runtime::Runtime, String> {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .map_err(|e| e.to_string())
}
