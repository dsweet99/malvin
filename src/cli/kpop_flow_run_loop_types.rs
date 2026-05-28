use crate::cli::KpopArgs;
use crate::prompts::PromptStore;

use crate::cli::kpop_flow::KpopPrepared;

pub(crate) struct RunKpopAgentLoopsParams<'a> {
    pub kpop: &'a KpopArgs,
    pub store: &'a PromptStore,
    pub client: &'a mut crate::acp::AgentClient,
    pub prepared: &'a KpopPrepared,
}
