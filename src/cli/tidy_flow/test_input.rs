#[cfg(test)]
use std::collections::HashMap;

#[cfg(test)]
use crate::acp::AgentClient;
#[cfg(test)]
use crate::artifacts::RunArtifacts;
#[cfg(test)]
use crate::prompts::PromptStore;

#[cfg(test)]
pub(crate) fn tidy_acp_input<'a>(
    client: &'a mut AgentClient,
    artifacts: &'a RunArtifacts,
    store: &'a PromptStore,
    context: &'a HashMap<String, String>,
) -> super::TidyAcpInput<'a> {
    super::TidyAcpInput {
        client,
        artifacts,
        store,
        context,
        run_learn: false,
        quick: false,
    }
}
