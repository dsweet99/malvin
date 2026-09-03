use std::path::PathBuf;

use crate::agent_backend::AgentBackend;
use crate::artifacts::RunArtifacts;
use crate::orchestrator::workflow_context_paths_only;
use crate::prompt_stratification::{PromptStratum, join_labeled_strata};
use crate::prompts::{DO_HEADER_MD, HEADER_MD, PromptError, PromptStore, render_header};

pub struct BindMalvinHeader<'a> {
    pub client: &'a mut AgentBackend,
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub model: &'a str,
    pub git: bool,
    pub log_path: PathBuf,
}

/// Bind rendered `header.md` so [`AgentBackend::start_coder_session`] delivers it
/// to a freshly created agent (router / write).
pub fn bind_malvin_header(input: BindMalvinHeader<'_>) -> Result<(), String> {
    let ctx = workflow_context_paths_only(input.artifacts, input.model, input.git);
    let prompt = render_header(input.store, ctx.as_map()).map_err(|e| e.0)?;
    input.client.bind_session_header(
        prompt.trim().to_string(),
        input.log_path,
        HEADER_MD,
    );
    Ok(())
}

/// Bind spawn header for `--do`: `do_header.md` (mode) with `header.md` context,
/// so both are delivered at spawn rather than on the work turn.
pub fn bind_do_header(input: BindMalvinHeader<'_>) -> Result<(), String> {
    let ctx = workflow_context_paths_only(input.artifacts, input.model, input.git);
    let coding = render_header(input.store, ctx.as_map()).map_err(|e| e.0)?;
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
    use crate::prompts::{DO_HEADER_MD, HEADER_MD, PromptStore};
    use crate::test_utils::with_isolated_home;

    #[test]
    fn kiss_cov_bind_malvin_header() {
        let _ = super::bind_malvin_header;
        let _ = super::bind_do_header;
        let _: Option<super::BindMalvinHeader<'_>> = None;
    }

    #[test]
    fn kiss_cov_bind_session_headers() {
        let _ = super::bind_malvin_header;
        let _ = super::bind_do_header;
    }

    #[test]
    fn bind_do_header_includes_header_and_do_header_at_spawn() {
        with_isolated_home(|work| {
            let artifacts = crate::artifacts::create_run_artifacts_from_text_opts(
                "req",
                Some(work),
                crate::run_id::RunDirOptions::default(),
            )
            .expect("artifacts");
            let prompt_root = artifacts.run_dir.join("prompts");
            std::fs::create_dir_all(&prompt_root).expect("mkdir");
            std::fs::write(prompt_root.join(HEADER_MD), "HDR\n").expect("header");
            std::fs::write(prompt_root.join(DO_HEADER_MD), "DO\n").expect("do_header");
            let store = PromptStore::with_root(prompt_root);
            let mut client = crate::agent_backend::agent_backend_from_client(
                crate::cursor_sdk::cursor_sdk_client_from_raw(
                    "cursor:auto",
                    crate::acp::AgentIoOptions {
                        force: true,
                        no_tee: true,
                        raw_output: true,
                        show_thoughts_on_stdout: false,
                        emit_stdout_markdown: false,
                        log_full_outgoing_prompts: false,
                    },
                    1,
                ),
            );
            bind_do_header(BindMalvinHeader {
                client: &mut client,
                store: &store,
                artifacts: &artifacts,
                model: crate::config::DEFAULT_CLI_MODEL,
                git: false,
                log_path: artifacts.log_path("do_header"),
            })
            .expect("bind");
            let header = client.session_header.as_ref().expect("bound");
            assert!(header.prompt.contains("HDR"));
            assert!(header.prompt.contains("DO"));
            assert_eq!(header.stdout_label, DO_HEADER_MD);
        });
    }
}
