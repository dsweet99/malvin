use std::collections::HashMap;

use malvin::artifacts::{RunArtifacts, SessionDotfileBackups};
use malvin::prompts::PromptStore;
use malvin::run_timing::TimingPhase;

#[must_use]
pub(crate) fn effective_tidy_max_loops(max_loops: usize) -> usize {
    max_loops.max(1)
}

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
        artifacts: RunArtifacts,
        session_dotfile_backups: SessionDotfileBackups,
    },
    RunAgent {
        client: malvin::AgentClient,
        artifacts: RunArtifacts,
        session_dotfile_backups: SessionDotfileBackups,
        store: PromptStore,
        context: HashMap<String, String>,
        run_learn: bool,
    },
}

pub struct TidyAcpInput<'a> {
    pub(crate) client: &'a mut malvin::AgentClient,
    pub(crate) artifacts: &'a RunArtifacts,
    pub(crate) store: &'a PromptStore,
    pub(crate) context: &'a HashMap<String, String>,
    pub(crate) run_learn: bool,
}

pub struct TidyPromptRestore<'a> {
    pub(crate) prompt: &'a str,
    pub(crate) label: &'a str,
    pub(crate) phase: TimingPhase,
    pub(crate) session_dotfile_backups: &'a SessionDotfileBackups,
    pub(crate) restore_context: &'a str,
}
