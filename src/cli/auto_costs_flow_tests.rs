use std::cell::{Cell, RefCell};
use std::path::Path;

use super::{
    CostSetupEnv, CostSetupOutcome, prepare_cost_setup, record_cost_setup_outcome,
    render_set_costs_request, router_for_cost_setup, should_setup_costs, skip_automatic_cost_setup,
};
use crate::config::malvin_config_file::{MalvinConfig, parse_malvin_config};
use crate::config::model_id::{ParsedModel, parse_model_id};
use crate::test_support::test_stderr_capture::capture_stderr_output;

fn cursor() -> ParsedModel {
    parse_model_id("cursor:auto").expect("model")
}

fn open_config() -> MalvinConfig {
    parse_malvin_config("auto_setup_costs = true")
}

fn priced_config() -> MalvinConfig {
    parse_malvin_config("[agent.cursor.auto]\nusd_per_microtoken_in = 1.0")
}

fn config(auto_setup_costs: bool) -> MalvinConfig {
    parse_malvin_config(&format!("auto_setup_costs = {auto_setup_costs}"))
}

fn env_for(model: &ParsedModel, disabled: bool) -> CostSetupEnv<'_> {
    CostSetupEnv {
        model,
        work_dir: Path::new("."),
        disabled,
    }
}

struct Seam {
    ran: bool,
    loads: usize,
    stderr: String,
}

fn apply_cost_setup<S, L>(env: &CostSetupEnv<'_>, run: S, mut load: L)
where
    S: FnOnce(&ParsedModel) -> CostSetupOutcome,
    L: FnMut(&Path) -> MalvinConfig,
{
    if !prepare_cost_setup(env, &mut load) {
        return;
    }
    record_cost_setup_outcome(env, run(env.model), &mut load);
}

fn drive(env: &CostSetupEnv<'_>, configs: Vec<MalvinConfig>, outcome: CostSetupOutcome) -> Seam {
    let loads = Cell::new(0_usize);
    let ran = Cell::new(false);
    let pending = RefCell::new(configs);
    let stderr = capture_stderr_output(|| {
        apply_cost_setup(
            env,
            |_| {
                ran.set(true);
                outcome
            },
            |_| {
                loads.set(loads.get() + 1);
                pending.borrow_mut().remove(0)
            },
        );
    });
    Seam {
        ran: ran.get(),
        loads: loads.get(),
        stderr,
    }
}

#[test]
fn rendered_request_names_the_model_and_forbids_repo_edits() {
    let text = render_set_costs_request(&cursor()).expect("render");
    assert!(text.contains("cursor:auto"));
    assert!(text.contains("~/.malvinconf/config.toml"));
    assert!(text.contains("Do not change repository files"));
    assert!(!text.contains("{{"));
}

#[test]
fn setup_is_guarded_by_switches_live_prices_and_configured_rates() {
    let cursor = cursor();
    assert!(should_setup_costs(&config(true), &cursor, false));
    assert!(!should_setup_costs(&config(false), &cursor, false));
    assert!(!should_setup_costs(&config(true), &cursor, true));

    let openrouter = parse_model_id("pi:openrouter/openai/gpt-4o-mini").expect("model");
    let portkey = parse_model_id("pi:portkey/openai/gpt-4o-mini").expect("model");
    assert!(!should_setup_costs(&config(true), &openrouter, false));
    assert!(!should_setup_costs(&config(true), &portkey, false));

    let priced = parse_malvin_config("[agent.cursor.auto]\nusd_per_microtoken_in = 1.0");
    assert!(!should_setup_costs(&priced, &cursor, false));
}

#[test]
fn failed_workflow_without_detail_uses_the_plain_warning() {
    let model = cursor();
    let seam = drive(
        &env_for(&model, false),
        vec![open_config(), priced_config()],
        CostSetupOutcome::Failed(String::new()),
    );
    assert!(seam.ran);
    assert!(
        seam.stderr.contains(
            "automatic token-cost setup did not complete; continuing with configured rates"
        )
    );
    assert!(!seam.stderr.contains("rates:"));
    assert!(!seam.stderr.contains("remain unset"));
}

#[test]
fn failed_workflow_reloads_config_and_warns() {
    let model = cursor();
    let seam = drive(
        &env_for(&model, false),
        vec![open_config(), open_config()],
        CostSetupOutcome::Failed("render failed".to_string()),
    );
    assert!(seam.ran);
    assert_eq!(seam.loads, 2);
    assert!(
        seam.stderr.contains(
            "automatic token-cost setup did not complete; continuing with configured rates: render failed"
        )
    );
    assert!(
        seam.stderr
            .contains("token costs remain unset after automatic setup")
    );
}

#[test]
fn completed_workflow_reloads_saved_rates_without_a_warning() {
    let model = cursor();
    let seam = drive(
        &env_for(&model, false),
        vec![open_config(), priced_config()],
        CostSetupOutcome::Completed,
    );
    assert!(seam.ran);
    assert_eq!(seam.loads, 2);
    assert_eq!(seam.stderr, "");
}

#[test]
fn completed_workflow_warns_when_reloaded_rates_stay_unset() {
    let model = cursor();
    let seam = drive(
        &env_for(&model, false),
        vec![open_config(), open_config()],
        CostSetupOutcome::Completed,
    );
    assert!(seam.ran);
    assert_eq!(seam.loads, 2);
    assert!(!seam.stderr.contains("did not complete"));
    assert!(
        seam.stderr
            .contains("token costs remain unset after automatic setup")
    );
}

#[test]
fn disabled_priced_and_live_models_do_not_run() {
    let cursor = cursor();
    let disabled = drive(
        &env_for(&cursor, true),
        vec![open_config()],
        CostSetupOutcome::Completed,
    );
    let priced = drive(
        &env_for(&cursor, false),
        vec![priced_config()],
        CostSetupOutcome::Completed,
    );
    let live = parse_model_id("pi:openrouter/openai/gpt-4o-mini").expect("model");
    let skipped = drive(
        &env_for(&live, false),
        vec![open_config()],
        CostSetupOutcome::Completed,
    );
    for seam in [&disabled, &priced, &skipped] {
        assert!(!seam.ran);
        assert_eq!(seam.loads, 1);
        assert_eq!(seam.stderr, "");
    }
}

#[test]
fn cost_setup_router_clears_harness_gates() {
    let mut router = crate::cli::RouterOpts::test_defaults();
    router.gates = true;
    let setup = router_for_cost_setup(&router);
    assert!(!setup.gates);
    assert!(router.gates);
    assert_eq!(setup.max_loops, router.max_loops);
}

#[test]
fn unit_tests_skip_the_real_cost_workflow() {
    assert!(skip_automatic_cost_setup());
}

#[test]
fn cost_setup_runs_in_process_router_before_init() {
    let src = include_str!("auto_costs.rs");
    assert!(
        src.contains("run_router"),
        "cost setup must call run_router"
    );
    assert!(
        src.contains("set_costs.md"),
        "cost setup must render set_costs.md"
    );
    assert!(
        !src.contains("current_exe"),
        "cost setup must not spawn malvin"
    );
    assert!(
        !src.contains("Command::"),
        "cost setup must not spawn a process"
    );
    let dispatch = include_str!("entrypoint_dispatch.rs");
    let async_at = dispatch.find("run_async_cli").expect("async runtime");
    let cost_at = dispatch
        .find("maybe_run_cost_setup")
        .expect("cost setup call");
    let init_at = dispatch
        .find("maybe_run_init_bootstrap")
        .expect("init bootstrap");
    assert!(cost_at > async_at);
    assert!(cost_at < init_at);
}
