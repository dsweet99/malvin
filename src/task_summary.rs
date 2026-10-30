mod model;
mod schedule;
mod text;
mod watch;
mod worker;
pub use worker::run_if_worker;

use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::artifacts::RunArtifacts;
use crate::config::model_list::ModelList;

pub struct SummaryTask(tokio::task::JoinHandle<()>);

impl Drop for SummaryTask {
    fn drop(&mut self) {
        self.0.abort();
    }
}

pub fn start(artifacts: &RunArtifacts) -> Option<SummaryTask> {
    let config = crate::config::malvin_config_file::load_malvin_config(&artifacts.work_dir);
    if config.agent.model_lite.is_none() {
        let request = std::fs::read_to_string(&artifacts.plan_path).ok()?;
        text::local_task(&request)?;
    }
    let artifacts = artifacts.clone();
    Some(SummaryTask(tokio::spawn(async move {
        if let Ok(child) = worker::spawn(&artifacts) {
            watch::until_exit(child, &artifacts.run_dir).await;
        }
    })))
}

pub(super) async fn generate_task(
    artifacts: &RunArtifacts,
    models: Option<&ModelList>,
) -> Result<(), String> {
    let request = tokio::fs::read_to_string(&artifacts.plan_path)
        .await
        .map_err(|e| e.to_string())?;
    if request.trim().is_empty() {
        return Err("request is empty".into());
    }
    let summary = if let Some(summary) = text::local_task(&request) {
        summary
    } else {
        let directions = text::directions("user request");
        let models = models.ok_or("model_lite is not configured")?;
        model::summarize(models, &artifacts.work_dir, &request, &directions).await?
    };
    store(&text::task_path(&artifacts.run_dir), &summary).await
}

pub(super) async fn generate_status(
    artifacts: &RunArtifacts,
    models: &ModelList,
) -> Result<(), String> {
    let Some(source) = status_source(&artifacts.run_dir) else {
        return Ok(());
    };
    let directions = text::directions("agent's current status");
    let summary = model::summarize(models, &artifacts.work_dir, &source, &directions).await?;
    store(&text::status_path(&artifacts.run_dir), &summary).await
}

fn status_source(run_dir: &Path) -> Option<String> {
    let mut file = std::fs::File::open(run_dir.join(crate::malvin_constants::STDOUT_LOG)).ok()?;
    let len = file.metadata().ok()?.len();
    if len == 0 {
        return None;
    }
    let start = len.saturating_sub(8_192);
    file.seek(SeekFrom::Start(start)).ok()?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).ok()?;
    let owned = String::from_utf8_lossy(&bytes).into_owned();
    let sliced = if start == 0 {
        owned
    } else {
        match owned.split_once('\n') {
            Some((_, rest)) => rest.to_string(),
            None => owned,
        }
    };
    text::status_excerpt(&sliced)
}

async fn store(path: &Path, summary: &str) -> Result<(), String> {
    let tmp = path.with_extension("tmp");
    tokio::fs::write(&tmp, summary)
        .await
        .map_err(|e| e.to_string())?;
    tokio::fs::rename(&tmp, path)
        .await
        .map_err(|e| e.to_string())
}

#[must_use]
pub fn current() -> String {
    stored_task().unwrap_or_default()
}

#[must_use]
pub fn metadata_tokens() -> Option<serde_json::Value> {
    let mut map = serde_json::Map::new();
    if let Some(task) = stored_task() {
        map.insert("task".into(), serde_json::Value::String(task));
    }
    if let Some(status) = stored_status() {
        map.insert("status".into(), serde_json::Value::String(status));
    }
    if map.is_empty() {
        None
    } else {
        Some(serde_json::Value::Object(map))
    }
}

fn stored_task() -> Option<String> {
    stored_file(text::task_path)
}

fn stored_status() -> Option<String> {
    stored_file(text::status_path)
}

fn stored_file(path: fn(&Path) -> std::path::PathBuf) -> Option<String> {
    let run = crate::workspace::run_id::active_run_dir()?;
    let text = std::fs::read_to_string(path(&run)).ok()?;
    text::publishable(&text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_direction_echo_does_not_become_a_token() {
        let _guard = crate::workspace::run_id::ACTIVE_RUN_DIR_TEST_LOCK
            .lock()
            .unwrap();
        let dir = tempfile::tempdir().unwrap();
        let run = dir.path().join("hash/run");
        std::fs::create_dir_all(&run).unwrap();
        crate::workspace::run_id::set_active_run_dir(Some(run.clone()));
        std::fs::write(text::task_path(&run), "Return only that summary").unwrap();
        assert_eq!(current(), "");
        assert!(metadata_tokens().is_none());
        crate::workspace::run_id::set_active_run_dir(None);
    }

    #[test]
    fn ready_summaries_follow_the_run_and_fill_both_tokens() {
        let _guard = crate::workspace::run_id::ACTIVE_RUN_DIR_TEST_LOCK
            .lock()
            .unwrap();
        let dir = tempfile::tempdir().unwrap();
        let run = dir.path().join("hash/run");
        std::fs::create_dir_all(&run).unwrap();
        crate::workspace::run_id::set_active_run_dir(Some(run.clone()));
        assert_eq!(current(), "");
        assert!(metadata_tokens().is_none());
        std::fs::write(text::task_path(&run), "Add cheap request summaries").unwrap();
        assert!(crate::time_format::heartbeat_payload_now()
            .ends_with("hash/run Add cheap request summaries"));
        std::fs::write(text::status_path(&run), "Reading the plan").unwrap();
        let tokens = metadata_tokens().unwrap();
        assert_eq!(tokens["task"], "Add cheap request summaries");
        assert_eq!(tokens["status"], "Reading the plan");
        crate::workspace::run_id::set_active_run_dir(None);
        assert_eq!(current(), "");
        assert!(metadata_tokens().is_none());
    }
}
