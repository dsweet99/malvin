use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use crate::artifacts::RunArtifacts;
use crate::config::model_list::ModelList;
use crate::self_worker::Worker;

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
    let log = open_log(log_path)?;
    let models = models.cli_value();
    let args = [prompt_path.as_os_str(), OsStr::new(&models)];
    let mut command = crate::self_worker::attached_command(Worker::FactAgent, args, log)?;
    command.current_dir(&artifacts.work_dir);
    Ok(command)
}

fn spawn_context(artifacts: &RunArtifacts) -> String {
    format!(
        "exe={:?}, cwd={}",
        crate::self_exe::self_exe().ok(),
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

pub fn run_worker(args: Vec<OsString>) -> Result<(), String> {
    let (prompt_path, models) = worker_args(args)?;
    run_prompt(prompt_path, &models)
}

fn worker_args(args: Vec<OsString>) -> Result<(PathBuf, String), String> {
    let usage = || crate::self_worker::usage(Worker::FactAgent, "PROMPT_PATH MODELS");
    let [prompt_path, models] = <[OsString; 2]>::try_from(args).map_err(|_| usage())?;
    let models = models.into_string().map_err(|_| usage())?;
    Ok((PathBuf::from(prompt_path), models))
}

fn run_prompt(prompt_path: PathBuf, models: &str) -> Result<(), String> {
    let models = crate::config::malvin_config_file::parse_model_list_cli_arg(models)?;
    let prompt = std::fs::read_to_string(&prompt_path)
        .map_err(|e| format!("cannot read {}: {e}", prompt_path.display()))?;
    let artifacts = worker_artifacts(prompt_path)?;
    let trace_path = fact_trace_path(&artifacts)?;
    let disconnected = crate::self_worker::parent_disconnected();
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

#[cfg(test)]
#[path = "fact_worker_tests.rs"]
mod tests;
