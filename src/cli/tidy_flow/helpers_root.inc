mod prep {
    include!("prep.inc");
}
mod prompt {
    include!("prompt.inc");
}
mod interleaved_loop {
    include!("interleaved_loop.inc");
}
pub(crate) mod recovery {
    include!("recovery.inc");
}
mod run {
    include!("run.inc");
}
mod run_startup {
    include!("run_startup.inc");
}

pub use prep::{
    compose_tidy_concerns_prompt, compose_tidy_prompt, prepare_tidy_prompt_store,
    write_checks_do_not_pass_for_artifacts, write_checks_do_not_pass_to_review_path,
};
pub use prompt::{run_tidy_prompt, run_tidy_prompt_with_restore};
pub use interleaved_loop::run_tidy_interleaved_loop;
pub use run::{merge_tidy_timing, run_tidy_acp};
pub use run_startup::{prepare_tidy_run, tidy_prompt_context};

pub enum TidyStartup {
    SkipAgent {
        artifacts: crate::artifacts::RunArtifacts,
        session_dotfile_backups: crate::artifacts::SessionDotfileBackups,
    },
    RunAgent {
        client: crate::acp::AgentClient,
        artifacts: crate::artifacts::RunArtifacts,
        session_dotfile_backups: crate::artifacts::SessionDotfileBackups,
        store: crate::prompts::PromptStore,
        context: std::collections::HashMap<String, String>,
        run_learn: bool,
    },
}

pub struct TidyAcpInput<'a> {
    pub(crate) client: &'a mut crate::acp::AgentClient,
    pub(crate) artifacts: &'a crate::artifacts::RunArtifacts,
    pub(crate) store: &'a crate::prompts::PromptStore,
    pub(crate) context: &'a std::collections::HashMap<String, String>,
    pub(crate) run_learn: bool,
}

pub struct TidyPromptRestore<'a> {
    pub(crate) prompt: &'a str,
    pub(crate) label: &'a str,
    pub(crate) phase: crate::run_timing::TimingPhase,
    pub(crate) session_dotfile_backups: &'a crate::artifacts::SessionDotfileBackups,
    pub(crate) restore_context: &'a str,
}
