use std::collections::HashMap;
use std::path::Path;

use crate::cli::router_flow::{RouterArgs, run_router};
use crate::cli::{RouterOpts, SharedOpts};
use crate::config::malvin_config_file::{MalvinConfig, TokenCostRates, load_malvin_config};
use crate::config::model_id::ParsedModel;
use malvin::prompts::{PromptError, PromptStore};

const SET_COSTS_PROMPT: &str = "set_costs.md";

fn rates_are_unset(rates: TokenCostRates) -> bool {
    rates == TokenCostRates::default()
}

fn model_has_live_costs(model: &ParsedModel) -> bool {
    model.pi_provider_and_model().is_some_and(|(provider, _)| {
        provider.eq_ignore_ascii_case("openrouter") || provider.eq_ignore_ascii_case("portkey")
    })
}

fn should_setup_costs(config: &MalvinConfig, model: &ParsedModel, disabled: bool) -> bool {
    !disabled
        && config.auto_setup_costs
        && !model_has_live_costs(model)
        && rates_are_unset(config.token_cost_rates_for(&model.canonical()))
}

struct CostSetupEnv<'a> {
    model: &'a ParsedModel,
    work_dir: &'a Path,
    disabled: bool,
}

enum CostSetupOutcome {
    Completed,
    Failed(String),
}

fn skip_automatic_cost_setup() -> bool {
    cfg!(test) || crate::agent_process::test_no_real_agent_enabled()
}

fn render_set_costs_request(model: &ParsedModel) -> Result<String, String> {
    let store = PromptStore::default_store();
    store.ensure_defaults().map_err(|e: PromptError| e.0)?;
    store
        .validate_exists(SET_COSTS_PROMPT)
        .map_err(|e: PromptError| e.0)?;
    let mut ctx = HashMap::new();
    ctx.insert("model_id".to_string(), model.canonical());
    store
        .render_prompt_only(SET_COSTS_PROMPT, &ctx)
        .map(|text| text.trim().to_string())
        .map_err(|e: PromptError| e.0)
}

fn warn_if_reloaded_rates_unset<L>(env: &CostSetupEnv<'_>, load: &mut L)
where
    L: FnMut(&Path) -> MalvinConfig,
{
    let refreshed = load(env.work_dir);
    if rates_are_unset(refreshed.token_cost_rates_for(&env.model.canonical())) {
        crate::output::print_log_warning("token costs remain unset after automatic setup");
    }
}

fn record_cost_setup_outcome<L>(env: &CostSetupEnv<'_>, outcome: CostSetupOutcome, mut load: L)
where
    L: FnMut(&Path) -> MalvinConfig,
{
    if let CostSetupOutcome::Failed(err) = &outcome {
        let line = if err.is_empty() {
            "automatic token-cost setup did not complete; continuing with configured rates"
                .to_string()
        } else {
            format!(
                "automatic token-cost setup did not complete; continuing with configured rates: {err}"
            )
        };
        crate::output::print_log_warning(&line);
    }
    warn_if_reloaded_rates_unset(env, &mut load);
}

fn prepare_cost_setup<L>(env: &CostSetupEnv<'_>, load: &mut L) -> bool
where
    L: FnMut(&Path) -> MalvinConfig,
{
    let config = load(env.work_dir);
    should_setup_costs(&config, env.model, env.disabled)
}

fn router_for_cost_setup(router: &RouterOpts) -> RouterOpts {
    crate::cli::init_flow::router_for_init_bootstrap(router)
}

async fn run_set_costs_workflow(shared: &SharedOpts, router: &RouterOpts) -> Result<(), String> {
    let request = render_set_costs_request(shared.model.first())?;
    let setup_router = router_for_cost_setup(router);
    run_router(
        RouterArgs {
            request: Some(request),
            max_loops: crate::cli::init_flow::effective_init_max_loops(router.max_loops),
        },
        crate::cli::AgentRouteOpts {
            shared,
            router: &setup_router,
        },
    )
    .await
}

pub(super) async fn maybe_run_cost_setup(shared: &SharedOpts, router: &RouterOpts) {
    if skip_automatic_cost_setup() {
        return;
    }
    let Ok(work_dir) = std::env::current_dir() else {
        return;
    };
    let env = CostSetupEnv {
        model: shared.model.first(),
        work_dir: &work_dir,
        disabled: shared.no_auto_costs,
    };
    if !prepare_cost_setup(&env, &mut load_malvin_config) {
        return;
    }
    let outcome = match run_set_costs_workflow(shared, router).await {
        Ok(()) => CostSetupOutcome::Completed,
        Err(err) => CostSetupOutcome::Failed(err),
    };
    record_cost_setup_outcome(&env, outcome, load_malvin_config);
}

#[cfg(test)]
#[path = "auto_costs_flow_tests.rs"]
mod auto_costs_flow_tests;
