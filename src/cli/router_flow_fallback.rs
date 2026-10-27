use super::router_flow::new_router_client;
use super::router_flow_loop::{RouterAgentLoopInput, run_router_agent_loops};
use malvin::agent_process::account_denial;
use malvin::run_timing::RunTiming;
use std::sync::{Arc, Mutex};

enum RouterModelError {
    Model(String),
    Workflow(String),
}

pub(super) async fn run_router_models(input: RouterAgentLoopInput<'_>) -> Result<(), String> {
    let mut prior_timing = None;
    let mut models = input.shared.model.iter().peekable();
    while let Some(model) = models.next() {
        let mut shared = input.shared.clone();
        shared.model = model.clone().into();
        let result = run_selected_model(
            &mut RouterAgentLoopInput {
                client: input.client,
                artifacts: input.artifacts,
                prompt_store: input.prompt_store,
                shared: &shared,
                router: input.router,
                max_loops: input.max_loops,
            },
            &mut prior_timing,
        )
        .await;
        match result {
            Err(RouterModelError::Model(error))
                if should_fallback(&error) && models.peek().is_some() =>
            {
                malvin::output::print_log_warning(&format!(
                    "{} failed due to insufficient funds; switching to {}: {error}",
                    model.canonical(),
                    models.peek().expect("next model").canonical(),
                ));
            }
            Ok(()) => return Ok(()),
            Err(RouterModelError::Model(error) | RouterModelError::Workflow(error)) => {
                return Err(error);
            }
        }
    }
    unreachable!("ModelList is nonempty")
}

fn should_fallback(error: &str) -> bool {
    model_quota_exhausted(error)
        && !error.contains(" restore:")
        && !error.starts_with("ABORT:")
        && !error.contains("workspace session restore failed")
}

fn model_quota_exhausted(error: &str) -> bool {
    account_denial(error).is_some()
}

async fn run_selected_model(
    input: &mut RouterAgentLoopInput<'_>,
    prior_timing: &mut Option<Arc<Mutex<RunTiming>>>,
) -> Result<(), RouterModelError> {
    *input.client = new_router_client(input.shared);
    input.client.set_run_timing(prior_timing.clone());
    input.client.prompts_log_run_dir = Some(input.artifacts.run_dir.clone());
    input
        .client
        .ensure_authenticated()
        .map_err(|e| RouterModelError::Model(e.to_string()))?;
    let outcome = run_router_agent_loops(RouterAgentLoopInput {
        client: input.client,
        artifacts: input.artifacts,
        prompt_store: input.prompt_store,
        shared: input.shared,
        router: input.router,
        max_loops: input.max_loops,
    })
    .await
    .map_err(RouterModelError::Workflow)?;
    *prior_timing = outcome.timing;
    malvin::acp_post_run::merge_acp_restore_and_check_abort(
        outcome.last_acp,
        input.artifacts,
        &outcome.last_backups,
    )
    .map_err(RouterModelError::Model)
}

#[cfg(test)]
#[path = "router_flow_fallback_tests.rs"]
mod tests;
