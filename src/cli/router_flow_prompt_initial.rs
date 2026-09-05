use crate::artifacts::RunArtifacts;
use crate::prompt_stratification::{
    AggregatedInitialPrompt, AggregatedInitialPromptBuilder,
};
use crate::prompts::{PromptStore, header_prompt_file};

use super::{
    RouterAPromptInput, RouterHeaderPromptInput, RouterKpopCommonPromptInput,
    build_router_a_prompt, build_router_header_prompt, build_router_kpop_common_prompt,
    build_router_mbc2_prompt, kpop_common_prompt_label, router_a_prompt_label,
};

/// Inputs for the aggregated first router turn (header + setup + `router_a`).
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct RouterInitialPromptInput<'a> {
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub model: &'a str,
    pub git: bool,
    pub gates: bool,
    pub no_kpop: bool,
    pub creative: bool,
    pub max_hypotheses: usize,
    /// When true, include `header.md` (fresh agent). When false, skip it (reused/resumed).
    pub include_header: bool,
}

/// Aggregated initial router prompt for one host send.
pub(crate) type RouterInitialPrompt = AggregatedInitialPrompt;

const ROUTER_INITIAL_LOG_WHO: &str = "router_initial";

/// Render and join the initial router pieces for the active workflow options.
///
/// Order: optional `header.md`, `kpop_common` (omitted when empty / `no_kpop`),
/// optional `mbc2.md` when creative, then `router_a`.
pub(crate) fn build_router_initial_prompt(
    input: RouterInitialPromptInput<'_>,
) -> Result<RouterInitialPrompt, String> {
    let mut builder = AggregatedInitialPromptBuilder::new();

    if input.include_header {
        let header = build_router_header_prompt(RouterHeaderPromptInput {
            store: input.store,
            artifacts: input.artifacts,
            model: input.model,
            git: input.git,
        })?;
        builder.push_nonempty(header_prompt_file(), header);
    }

    let kpop = build_router_kpop_common_prompt(RouterKpopCommonPromptInput {
        store: input.store,
        artifacts: input.artifacts,
        model: input.model,
        git: input.git,
        max_hypotheses: input.max_hypotheses,
        no_kpop: input.no_kpop,
    })?;
    builder.push_nonempty(kpop_common_prompt_label(input.no_kpop), kpop);

    if input.creative {
        let mbc2 = build_router_mbc2_prompt(input.store, input.artifacts)?;
        builder.push_nonempty("mbc2.md", mbc2);
    }

    let router_a = build_router_a_prompt(RouterAPromptInput {
        store: input.store,
        artifacts: input.artifacts,
        model: input.model,
        git: input.git,
        gates: input.gates,
        no_kpop: input.no_kpop,
    })?;
    builder.push_nonempty(router_a_prompt_label(input.no_kpop), router_a);

    Ok(builder.finish(ROUTER_INITIAL_LOG_WHO))
}

#[cfg(test)]
#[path = "router_flow_prompt_initial_tests.rs"]
mod router_flow_prompt_initial_tests;
