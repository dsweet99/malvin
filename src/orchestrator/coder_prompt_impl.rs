impl Orchestrator<'_> {
    pub(super) async fn run_coder_prompt(
        &mut self,
        filename: &str,
        context: &HashMap<String, String>,
        suffix: &str,
        llm_phase: TimingPhase,
    ) -> Result<(), WorkflowError> {
        let prompt = self
            .prompts
            .render(filename, context)
            .map_err(|e| WorkflowError(e.0))?;
        self.run_coder_prompt_body(prompt, filename, suffix, llm_phase)
            .await
    }

    pub(super) async fn run_coder_prompt_body(
        &mut self,
        prompt: String,
        filename: &str,
        suffix: &str,
        llm_phase: TimingPhase,
    ) -> Result<(), WorkflowError> {
        let stem = prompt_md_stem(filename);
        let log = self.artifacts.log_path(&format!("coder_{stem}_{suffix}"));
        let run_result = self
            .client
            .run_coder_prompt(
                &prompt,
                &log,
                stem,
                CoderPromptOptions {
                    llm_phase: Some(llm_phase),
                    skip_repo_style: false,
                    do_trace_split: None,
                    stdout_bracket_label: Some(filename),
                },
            )
            .await
            .map_err(|e: AgentError| WorkflowError(e.0));
        let restore_result = crate::artifacts::restore_workspace_session_dotfiles(
            &self.artifacts.work_dir,
            &self.session_dotfile_backups,
        )
        .map_err(WorkflowError);

        match (run_result, restore_result) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(run_err), Ok(())) => Err(run_err),
            (Ok(()), Err(restore_err)) => Err(restore_err),
            (Err(run_err), Err(restore_err)) => {
                Err(WorkflowError(format!("{}, {}", run_err.0, restore_err.0)))
            }
        }
    }
}
