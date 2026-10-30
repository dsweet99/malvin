mod worker;
pub use worker::run_if_worker;

use std::path::{Path, PathBuf};

use crate::artifacts::RunArtifacts;
use crate::config::model_id::ParsedModel;
use crate::config::model_list::ModelList;

pub struct SummaryTask(tokio::task::JoinHandle<()>);

impl Drop for SummaryTask {
    fn drop(&mut self) {
        self.0.abort();
    }
}

pub fn start(artifacts: &RunArtifacts) -> Option<SummaryTask> {
    let config = crate::config::malvin_config_file::load_malvin_config(&artifacts.work_dir);
    config.agent.model_lite.as_ref()?;
    let artifacts = artifacts.clone();
    Some(SummaryTask(tokio::spawn(async move {
        if worker::launch(&artifacts).await.is_ok() {
            crate::herdr::notify_summary_ready(&artifacts.run_dir);
        }
    })))
}

const DIRECTIONS: &str = "\
Summarize the user request between the markers in exactly five words. \
Return only those five words, on one line. \
The words must describe the request, not these directions.\n\n---\n";

async fn generate(artifacts: &RunArtifacts, models: &ModelList) -> Result<(), String> {
    let request = tokio::fs::read_to_string(&artifacts.plan_path)
        .await
        .map_err(|e| e.to_string())?;
    if request.trim().is_empty() {
        return Err("request is empty".into());
    }
    let prompt = format!("{DIRECTIONS}{request}\n---\n");
    let text = generate_with_fallback(models, &artifacts.work_dir, &prompt).await?;
    let summary = accept_summary(&request, &text).ok_or("summary echoes the directions")?;
    tokio::fs::write(summary_path(&artifacts.run_dir), summary)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

async fn generate_with_fallback(
    models: &ModelList,
    work_dir: &Path,
    prompt: &str,
) -> Result<String, String> {
    let mut last_error = String::new();
    for model in models {
        match generate_one(model, work_dir, prompt).await {
            Ok(text) => return normalize(&text).ok_or_else(|| "expected five words".into()),
            Err(error) => {
                if crate::agent_process::account_denial(&error).is_none() {
                    return Err(error);
                }
                last_error = error;
            }
        }
    }
    Err(last_error)
}

async fn generate_one(model: &ParsedModel, cwd: &Path, prompt: &str) -> Result<String, String> {
    use crate::backends::agent_backend::backend_lifecycle::backend_lifecycle;
    let backend = backend_lifecycle(model.backend);
    backend
        .ensure_authenticated(model)
        .map_err(|e| e.to_string())?;
    let session = backend
        .spawn_session(
            crate::backends::bridge_sdk::BridgeSpawnArgs {
                cwd,
                model,
                thinking: model.thinking_param(),
                io: crate::agent_process::AgentIoOptions {
                    no_tee: true,
                    raw_output: false,
                    show_thoughts_on_stdout: false,
                    emit_stdout_markdown: false,
                    log_full_outgoing_prompts: false,
                },
                run_dir: None,
                timing: None,
            },
            None,
            model.service_param(),
        )
        .await
        .map_err(|e| e.to_string())?;
    let result = session.send_prompt(prompt).await.map_err(|e| e.to_string());
    let text = session.last_text();
    let _ = session.shutdown().await;
    result.map(|()| text)
}

fn normalize(text: &str) -> Option<String> {
    let text = text.trim().trim_matches(['"', '\'']);
    if text.chars().any(|c| c.is_control() && !c.is_whitespace()) {
        return None;
    }
    let words: Vec<_> = text.split_whitespace().collect();
    if words.len() != 5 || only_directions(text) {
        return None;
    }
    Some(words.join(" "))
}

fn accept_summary(request: &str, text: &str) -> Option<String> {
    let summary = normalize(text)?;
    if task_echo(&summary) && !task_echo(request) {
        None
    } else {
        Some(summary)
    }
}

fn word_key(word: &str) -> String {
    word.trim_matches(|c: char| !c.is_alphanumeric())
        .to_ascii_lowercase()
}

fn only_directions(text: &str) -> bool {
    let mut any = false;
    for word in text.split_whitespace() {
        let key = word_key(word);
        if key.is_empty() {
            continue;
        }
        any = true;
        if !direction_word(&key) {
            return false;
        }
    }
    any
}

fn direction_word(key: &str) -> bool {
    DIRECTIONS
        .split_whitespace()
        .any(|word| word_key(word) == key)
}

fn task_echo(text: &str) -> bool {
    let keys: Vec<_> = text.split_whitespace().map(word_key).collect();
    let five = keys
        .iter()
        .any(|key| key == "five" || key.starts_with("five-"));
    let word = keys.iter().any(|key| {
        key == "word" || key == "words" || key.ends_with("-word") || key.ends_with("-words")
    });
    five && word
}

fn summary_path(run_dir: &Path) -> PathBuf {
    run_dir.join("request_summary.txt")
}

#[must_use]
pub fn current() -> String {
    crate::workspace::run_id::active_run_dir()
        .and_then(|run| std::fs::read_to_string(summary_path(&run)).ok())
        .and_then(|text| normalize(&text))
        .unwrap_or_default()
}

#[must_use]
pub fn herdr_status() -> String {
    let summary = current();
    if summary.is_empty() {
        "malvin".into()
    } else {
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_requires_five_words_and_removes_terminal_controls() {
        assert_eq!(
            normalize("\"Add cheap request summary models\""),
            Some("Add cheap request summary models".into())
        );
        assert_eq!(
            normalize("Add\ncheap request summary models"),
            Some("Add cheap request summary models".into())
        );
        assert_eq!(normalize("too short"), None);
        assert_eq!(normalize("one two three four five six"), None);
        assert_eq!(normalize("one two three four \x1b[31mfive"), None);
        assert_eq!(normalize(""), None);
        assert_eq!(normalize("Summarize request in five words."), None);
        assert_eq!(normalize("Summarize request in five words"), None);
    }

    #[test]
    fn summary_must_describe_the_request_rather_than_the_directions() {
        assert_eq!(
            accept_summary("Thanks", "User asks for five-word summary."),
            None
        );
        assert_eq!(
            accept_summary("Thanks", "\"User expresses thanks to assistant\""),
            Some("User expresses thanks to assistant".into())
        );
        assert_eq!(
            accept_summary(
                "Please implement request summaries.",
                "Add cheap request summary models"
            ),
            Some("Add cheap request summary models".into())
        );
        assert_eq!(
            accept_summary(
                "Reproduce herdr five-word summarization bug",
                "Reproduce herdr five-word summarization bug"
            ),
            Some("Reproduce herdr five-word summarization bug".into())
        );
        assert_eq!(
            accept_summary("Draft the Q3 report", "Draft the quarterly summary report"),
            Some("Draft the quarterly summary report".into())
        );
    }

    #[test]
    fn stored_direction_echo_does_not_become_herdr_status() {
        let _guard = crate::workspace::run_id::ACTIVE_RUN_DIR_TEST_LOCK
            .lock()
            .unwrap();
        let dir = tempfile::tempdir().unwrap();
        let run = dir.path().join("hash/run");
        std::fs::create_dir_all(&run).unwrap();
        crate::workspace::run_id::set_active_run_dir(Some(run.clone()));
        std::fs::write(summary_path(&run), "Summarize request in five words.").unwrap();
        assert_eq!(current(), "");
        assert_eq!(herdr_status(), "malvin");
        crate::workspace::run_id::set_active_run_dir(None);
    }

    #[test]
    fn pending_summary_uses_fallbacks_and_ready_summary_follows_id() {
        let _guard = crate::workspace::run_id::ACTIVE_RUN_DIR_TEST_LOCK
            .lock()
            .unwrap();
        let dir = tempfile::tempdir().unwrap();
        let run = dir.path().join("hash/run");
        std::fs::create_dir_all(&run).unwrap();
        crate::workspace::run_id::set_active_run_dir(Some(run.clone()));
        assert_eq!(current(), "");
        assert_eq!(herdr_status(), "malvin");
        std::fs::write(summary_path(&run), "Add cheap request summary models").unwrap();
        assert!(
            crate::time_format::heartbeat_payload_now()
                .ends_with("hash/run Add cheap request summary models")
        );
        assert_eq!(herdr_status(), "Add cheap request summary models");
        crate::workspace::run_id::set_active_run_dir(None);
        assert_eq!(current(), "");
    }
}
