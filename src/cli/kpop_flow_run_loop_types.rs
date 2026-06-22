use super::super::kpop_flow_a::KpopPrepared;
use crate::cli::args::KpopArgs;
use crate::prompts::PromptStore;

pub(crate) struct RunKpopAgentLoopsParams<'a> {
    pub kpop: &'a KpopArgs,
    pub store: &'a PromptStore,
    pub client: &'a mut crate::agent_backend::AgentBackend,
    pub prepared: &'a KpopPrepared,
}
#[cfg(test)]
#[path = "kpop_flow_run_loop_types_test.rs"]
mod kpop_flow_run_loop_types_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<RunKpopAgentLoopsParams> = None;
    }
}
