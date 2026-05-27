use crate::cli::{KpopArgs, SharedOpts, WorkflowCliOptions};
use crate::prompts::PromptStore;

use crate::cli::kpop_flow::KpopPrepared;

pub(crate) struct RunKpopAgentLoopsParams<'a> {
    pub kpop: &'a KpopArgs,
    #[allow(dead_code)]
    pub shared: &'a SharedOpts,
    #[allow(dead_code)]
    pub workflow: WorkflowCliOptions,
    pub store: &'a PromptStore,
    pub client: &'a mut crate::acp::AgentClient,
    pub prepared: &'a KpopPrepared,
}
