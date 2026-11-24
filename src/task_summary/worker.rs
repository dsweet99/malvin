use std::process::Stdio;
use std::time::Instant;

use crate::artifacts::RunArtifacts;
use crate::config::model_list::ModelList;

const WORKER_ARG: &str = "__summary-worker";

pub(super) fn spawn(artifacts: &RunArtifacts) -> Result<tokio::process::Child, String> {
    let mut command = worker_command(artifacts)?;
    command.kill_on_drop(true);
    command.spawn().map_err(|e| e.to_string())
}

fn worker_command(artifacts: &RunArtifacts) -> Result<tokio::process::Command, String> {
    let exe = crate::self_exe::self_exe().map_err(|e| e.to_string())?;
    let log = std::fs::File::create(artifacts.run_dir.join("request_summary.log"))
        .map_err(|e| e.to_string())?;
    let mut command = tokio::process::Command::new(exe);
    command
        .arg(WORKER_ARG)
        .arg(&artifacts.plan_path)
        .current_dir(&artifacts.work_dir)
        .env("MALVIN_NO_HERDR", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(log);
    crate::herdr::strip_herdr_env_tokio(&mut command);
    Ok(command)
}

#[must_use]
pub fn run_if_worker() -> Option<crate::Exit> {
    let result = worker_request(std::env::args_os().skip(1))?.and_then(run);
    Some(match result {
        Ok(()) => crate::Exit::Success,
        Err(error) => {
            eprintln!("{error}");
            crate::Exit::Failure
        }
    })
}

fn worker_request(
    mut args: impl Iterator<Item = std::ffi::OsString>,
) -> Option<Result<std::path::PathBuf, String>> {
    if args.next()? != WORKER_ARG {
        return None;
    }
    Some(
        args.next()
            .map(std::path::PathBuf::from)
            .ok_or_else(|| format!("usage: malvin {WORKER_ARG} REQUEST_PATH")),
    )
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
    let disconnected = crate::self_exe::parent_disconnected();
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

#[cfg(test)]
mod tests {
    use super::{WORKER_ARG, worker_request};
    use std::ffi::OsString;

    fn args(items: &[&str]) -> impl Iterator<Item = OsString> {
        items
            .iter()
            .map(OsString::from)
            .collect::<Vec<_>>()
            .into_iter()
    }

    #[test]
    fn worker_is_selected_only_by_argv_marker() {
        assert!(worker_request(args(&["--help"])).is_none());
        assert!(worker_request(args(&[])).is_none());
        let path = worker_request(args(&[WORKER_ARG, "/tmp/plan.md"])).unwrap();
        assert_eq!(path.unwrap(), std::path::PathBuf::from("/tmp/plan.md"));
        assert!(worker_request(args(&[WORKER_ARG])).unwrap().is_err());
    }
}
