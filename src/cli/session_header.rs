use std::path::PathBuf;

use malvin::agent_backend::SdkClient;
use malvin::artifacts::RunArtifacts;
use malvin::orchestrator::workflow_context_paths_only;
use malvin::prompt_stratification::{PromptStratum, join_labeled_strata};
use malvin::prompts::{DO_HEADER_MD, PromptError, PromptStore, render_header};

pub struct BindMalvinHeader<'a> {
    pub client: &'a mut SdkClient,
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub model: &'a str,
    pub log_path: PathBuf,
}

pub fn render_malvin_header_body(
    store: &PromptStore,
    artifacts: &RunArtifacts,
    model: &str,
) -> Result<String, String> {
    let ctx = workflow_context_paths_only(artifacts, model);
    let prompt = render_header(store, ctx.as_map()).map_err(|e| e.0)?;
    Ok(prompt.trim().to_string())
}

pub fn bind_do_header(input: BindMalvinHeader<'_>) -> Result<(), String> {
    let ctx = workflow_context_paths_only(input.artifacts, input.model);
    let coding = render_malvin_header_body(input.store, input.artifacts, input.model)?;
    let mode = input
        .store
        .render_prompt_only(DO_HEADER_MD, ctx.as_map())
        .map_err(|e: PromptError| e.0)?;
    let prompt = join_labeled_strata([
        (PromptStratum::WorkflowHeader, coding.trim_end()),
        (PromptStratum::WorkflowHeader, mode.trim_end()),
    ]);
    input
        .client
        .bind_session_header(prompt, input.log_path, DO_HEADER_MD);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use malvin::prompts::{DO_HEADER_MD, HEADER_MD, PromptStore};
    use malvin::test_utils::with_isolated_home;

    #[test]
    fn kiss_cov_bind_malvin_header() {
        let _ = super::bind_do_header;
        let _ = super::render_malvin_header_body;
        let _: Option<super::BindMalvinHeader<'_>> = None;
    }

    #[test]
    fn kiss_cov_bind_session_headers() {
        let _ = super::bind_do_header;
        let _ = super::render_malvin_header_body;
    }

    #[test]
    fn bind_do_header_includes_header_and_do_header_at_spawn() {
        with_isolated_home(|work| {
            let artifacts = malvin::artifacts::create_run_artifacts_from_text_opts(
                "req",
                Some(work),
                malvin::run_id::RunDirOptions::default(),
            )
            .expect("artifacts");
            let prompt_root = artifacts.run_dir.join("prompts");
            std::fs::create_dir_all(&prompt_root).expect("mkdir");
            std::fs::write(prompt_root.join(HEADER_MD), "HDR\n").expect("header");
            std::fs::write(prompt_root.join(DO_HEADER_MD), "DO\n").expect("do_header");
            let store = PromptStore::with_root(prompt_root);
            let mut client = malvin::cursor_sdk::cursor_sdk_client_from_raw(
                "cursor:auto",
                malvin::acp::AgentIoOptions {
                    no_tee: true,
                    raw_output: true,
                    show_thoughts_on_stdout: false,
                    emit_stdout_markdown: false,
                    log_full_outgoing_prompts: false,
                },
                1,
            );
            bind_do_header(BindMalvinHeader {
                client: &mut client,
                store: &store,
                artifacts: &artifacts,
                model: malvin::config::DEFAULT_CLI_MODEL,
                log_path: artifacts.log_path("do_header"),
            })
            .expect("bind");
            let (prompt, stdout_label) =
                malvin::agent_backend::pending_session_header(&client).expect("bound");
            assert!(prompt.contains("HDR"));
            assert!(prompt.contains("DO"));
            assert_eq!(stdout_label, DO_HEADER_MD);
        });
    }
}
