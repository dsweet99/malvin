use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use crate::artifacts::RunArtifacts;
use crate::config::model_list::ModelList;

const WORKER_ARG: &str = "__fact-agent-worker";
const PROMPT_FILE: &str = "fact_agent_prompt.md";
const LOG_FILE: &str = "fact_agent.log";

pub async fn run(models: &ModelList, artifacts: &RunArtifacts, prompt: &str) -> Result<(), String> {
    let prompt_path = artifacts.run_dir.join(PROMPT_FILE);
    let log_path = artifacts.run_dir.join(LOG_FILE);
    tokio::fs::write(&prompt_path, prompt)
        .await
        .map_err(|e| format!("cannot write {}: {e}", prompt_path.display()))?;
    let mut child = worker_command(models, artifacts, &prompt_path, &log_path)?
        .spawn()
        .map_err(|e| {
            format!(
                "cannot start fact agent worker: {e} ({})",
                spawn_context(artifacts)
            )
        })?;
    let _stdin = child.stdin.take();
    let status = child.wait().await.map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("worker {status}; see {}", log_path.display()))
    }
}

fn worker_command(
    models: &ModelList,
    artifacts: &RunArtifacts,
    prompt_path: &Path,
    log_path: &Path,
) -> Result<tokio::process::Command, String> {
    let exe = worker_exe()?;
    let log = open_log(log_path)?;
    let mut command = tokio::process::Command::new(exe);
    command
        .arg(WORKER_ARG)
        .arg(prompt_path)
        .arg(models.cli_value())
        .current_dir(&artifacts.work_dir)
        .env("MALVIN_NO_HERDR", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(log)
        .kill_on_drop(true);
    crate::herdr::strip_herdr_env_tokio(&mut command);
    Ok(command)
}

fn worker_exe() -> Result<PathBuf, String> {
    crate::self_exe::self_exe().map_err(|e| e.to_string())
}

fn spawn_context(artifacts: &RunArtifacts) -> String {
    format!(
        "exe={:?}, cwd={}",
        worker_exe().ok(),
        artifacts.work_dir.display()
    )
}

fn open_log(log_path: &Path) -> Result<std::fs::File, String> {
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .map_err(|e| format!("cannot open {}: {e}", log_path.display()))
}

#[must_use]
pub fn run_if_worker() -> Option<crate::Exit> {
    let result = worker_args(std::env::args_os().skip(1))?
        .and_then(|(prompt_path, models)| run_worker(prompt_path, &models));
    Some(match result {
        Ok(()) => crate::Exit::Success,
        Err(error) => {
            eprintln!("fact agent failed: {error}");
            crate::Exit::Failure
        }
    })
}

fn worker_args(
    mut args: impl Iterator<Item = std::ffi::OsString>,
) -> Option<Result<(PathBuf, String), String>> {
    if args.next()? != WORKER_ARG {
        return None;
    }
    let prompt_path = args.next().map(PathBuf::from);
    let models = args.next().and_then(|m| m.into_string().ok());
    Some(match (prompt_path, models) {
        (Some(path), Some(models)) => Ok((path, models)),
        _ => Err(format!("usage: malvin {WORKER_ARG} PROMPT_PATH MODELS")),
    })
}

fn run_worker(prompt_path: PathBuf, models: &str) -> Result<(), String> {
    let models = crate::config::malvin_config_file::parse_model_list_cli_arg(models)?;
    let prompt = std::fs::read_to_string(&prompt_path)
        .map_err(|e| format!("cannot read {}: {e}", prompt_path.display()))?;
    let artifacts = worker_artifacts(prompt_path)?;
    let trace_path = fact_trace_path(&artifacts)?;
    let disconnected = parent_disconnected();
    crate::task_summary::worker_runtime()?.block_on(async {
        tokio::select! {
            _ = disconnected => Err("parent malvin exited".to_string()),
            reply = crate::task_summary::run_background_prompt(&models, &artifacts, &prompt, &trace_path) => {
                eprintln!("fact agent replied: {}", reply?.trim());
                Ok(())
            }
        }
    })
}

fn worker_artifacts(prompt_path: PathBuf) -> Result<RunArtifacts, String> {
    let run_dir = prompt_path
        .parent()
        .ok_or("prompt path has no parent directory")?
        .to_path_buf();
    Ok(RunArtifacts {
        run_dir,
        plan_path: prompt_path,
        work_dir: std::env::current_dir().map_err(|e| e.to_string())?,
    })
}

fn fact_trace_path(artifacts: &RunArtifacts) -> Result<PathBuf, String> {
    let trace_path = crate::artifacts::AgentDirs(artifacts).fact_trace();
    if let Some(dir) = trace_path.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    }
    Ok(trace_path)
}

fn parent_disconnected() -> tokio::sync::oneshot::Receiver<()> {
    let (closed, disconnected) = tokio::sync::oneshot::channel();
    std::thread::spawn(move || {
        let _ = std::io::stdin().read_to_end(&mut Vec::new());
        let _ = closed.send(());
    });
    disconnected
}

#[cfg(test)]
#[path = "fact_worker_tests.rs"]
mod tests;
