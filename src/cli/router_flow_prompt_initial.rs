use malvin::artifacts::RunArtifacts;
use malvin::prompt_stratification::{AggregatedInitialPrompt, AggregatedInitialPromptBuilder};
use malvin::prompts::{PromptStore, header_prompt_file};

use super::{
    RouterAPromptInput, RouterHeaderPromptInput, RouterKpopCommonPromptInput,
    build_router_a_prompt, build_router_header_prompt, build_router_kpop_common_prompt,
    build_router_mbc2_prompt, router_a_prompt_label,
};
use malvin::prompts::kpop_common_prompt_file;

#[allow(clippy::struct_excessive_bools)]
pub(crate) struct RouterInitialPromptInput<'a> {
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub model: &'a str,
    pub gates: bool,
    pub no_kpop: bool,
    pub creative: bool,
    pub max_hypotheses: usize,
    pub include_header: bool,
    pub gate_iteration: usize,
}

pub(crate) type RouterInitialPrompt = AggregatedInitialPrompt;

const ROUTER_INITIAL_LOG_WHO: &str = "router_initial";

pub(crate) fn build_router_initial_prompt(
    input: RouterInitialPromptInput<'_>,
) -> Result<RouterInitialPrompt, String> {
    let mut builder = AggregatedInitialPromptBuilder::new();

    if input.include_header {
        let header = build_router_header_prompt(RouterHeaderPromptInput {
            store: input.store,
            artifacts: input.artifacts,
            model: input.model,
            max_hypotheses: input.max_hypotheses,
            no_kpop: input.no_kpop,
            gate_iteration: input.gate_iteration,
        })?;
        builder.push_nonempty(header_prompt_file(), header);
    } else if !input.no_kpop {
        let kpop = build_router_kpop_common_prompt(RouterKpopCommonPromptInput {
            store: input.store,
            artifacts: input.artifacts,
            model: input.model,
            max_hypotheses: input.max_hypotheses,
            no_kpop: input.no_kpop,
            gate_iteration: input.gate_iteration,
        })?;
        builder.push_nonempty(kpop_common_prompt_file(false), kpop);
    }

    if input.creative {
        let mbc2 = build_router_mbc2_prompt(input.store, input.artifacts)?;
        builder.push_nonempty("mbc2.md", mbc2);
    }

    let router_a = build_router_a_prompt(RouterAPromptInput {
        store: input.store,
        artifacts: input.artifacts,
        model: input.model,
        gates: input.gates,
        no_kpop: input.no_kpop,
    })?;
    builder.push_nonempty(router_a_prompt_label(input.no_kpop), router_a);

    Ok(builder.finish(ROUTER_INITIAL_LOG_WHO))
}

#[cfg(test)]
#[path = "router_flow_prompt_initial_tests.rs"]
mod router_flow_prompt_initial_tests;
