use malvin::artifacts::RunArtifacts;
use malvin::config::model_list::ModelList;
use malvin::prompts::{NEW_FACT_MD, PromptStore, TIDY_FACTS_MD};
use malvin::slow_task_watch::{
    SLOW_TASK_POLL, SLOW_TASK_THRESHOLD, SlowTaskWatch, WatchedPromptFlag, start,
};
use std::time::Duration;

use super::fact_agent_throttle;
use super::router_flow_prompt_turns::{RouterHeaderPromptInput, build_router_header_prompt};

pub(crate) struct FactAgentSpec<'a> {
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub models: &'a ModelList,
}

const NEW_FACT_PERCENT: usize = 25;

pub(crate) const fn pick_fact_prompt(roll: usize) -> &'static str {
    if roll < NEW_FACT_PERCENT {
        NEW_FACT_MD
    } else {
        TIDY_FACTS_MD
    }
}

pub(crate) fn build_fact_agent_prompt(
    store: &PromptStore,
    artifacts: &RunArtifacts,
    model: &str,
    prompt_name: &str,
) -> Result<String, String> {
    let header = build_router_header_prompt(RouterHeaderPromptInput {
        store,
        artifacts,
        model,
    })?;
    let fact = store
        .render_prompt_only(prompt_name, &std::collections::HashMap::new())
        .map_err(|e| e.0)?;
    Ok(format!("{header}\n\n{}", fact.trim()))
}

fn fact_agent_prompt(
    store: &PromptStore,
    artifacts: &RunArtifacts,
    model: &str,
) -> Result<String, String> {
    let name = pick_fact_prompt(fastrand::usize(..100));
    build_fact_agent_prompt(store, artifacts, model, name)
}

pub(crate) fn start_fact_agent_watch(
    spec: &FactAgentSpec<'_>,
    flag: &WatchedPromptFlag,
) -> SlowTaskWatch {
    start_fact_agent_watch_with(spec, flag, (SLOW_TASK_THRESHOLD, SLOW_TASK_POLL))
}

pub(crate) fn start_fact_agent_watch_with(
    spec: &FactAgentSpec<'_>,
    flag: &WatchedPromptFlag,
    timing: (Duration, Duration),
) -> SlowTaskWatch {
    let store = spec.store.clone();
    let models = spec.models.clone();
    let artifacts = spec.artifacts.clone();
    start(timing, flag.clone(), move || {
        let claimed = fact_agent_throttle::try_claim(
            &fact_agent_throttle::record_path(),
            malvin::clock::unix_now_secs(),
        );
        if !claimed {
            return;
        }
        tokio::spawn(async move {
            let model = models.first().canonical();
            let result = match fact_agent_prompt(&store, &artifacts, &model) {
                Ok(prompt) => malvin::fact_worker::run(&models, &artifacts, &prompt).await,
                Err(e) => Err(e),
            };
            if let Err(e) = result {
                malvin::output::print_log_error(&format!("fact agent failed: {e}"));
            }
        });
    })
}

#[cfg(test)]
#[path = "router_flow_fact_agent_tests.rs"]
mod router_flow_fact_agent_tests;
