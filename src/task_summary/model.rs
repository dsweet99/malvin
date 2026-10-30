use std::path::Path;

use crate::config::model_id::ParsedModel;
use crate::config::model_list::ModelList;

use super::text;

pub(super) async fn summarize(
    models: &ModelList,
    work_dir: &Path,
    source: &str,
    directions: &str,
) -> Result<String, String> {
    let prompt = format!("{directions}{source}\n---\n");
    let text = generate_with_fallback(models, work_dir, &prompt).await?;
    text::accept(source, &text, directions).map_err(str::to_string)
}

async fn generate_with_fallback(
    models: &ModelList,
    work_dir: &Path,
    prompt: &str,
) -> Result<String, String> {
    let mut last_error = String::new();
    for model in models {
        match generate_one(model, work_dir, prompt).await {
            Ok(text) => return Ok(text),
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
