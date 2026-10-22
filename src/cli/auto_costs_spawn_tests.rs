use std::cell::{Cell, RefCell};
use std::path::Path;

use super::{
    CostSetupEnv, CostSetupSpawn, command_for_cost_setup, cost_setup_args, cost_setup_child,
    drive_cost_setup,
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

fn env_for(model: &ParsedModel, disabled: bool) -> CostSetupEnv<'_> {
    CostSetupEnv {
        model,
        work_dir: Path::new("."),
        disabled,
    }
}

struct Seam {
    args: Vec<Vec<String>>,
    loads: usize,
    stderr: String,
}

fn drive(env: &CostSetupEnv<'_>, configs: Vec<MalvinConfig>, outcome: CostSetupSpawn) -> Seam {
    let loads = Cell::new(0_usize);
    let args = RefCell::new(Vec::new());
    let pending = RefCell::new(configs);
    let stderr = capture_stderr_output(|| {
        drive_cost_setup(
            env,
            |model| {
                args.borrow_mut().push(cost_setup_args(model));
                outcome
            },
            |_| {
                loads.set(loads.get() + 1);
                pending.borrow_mut().remove(0)
            },
        );
    });
    Seam {
        args: args.into_inner(),
        loads: loads.get(),
        stderr,
    }
}

#[test]
fn child_command_carries_canonical_model_and_recursion_guard() {
    let model = cursor();
    let child = cost_setup_child(&model).expect("current exe");
    let cmd = command_for_cost_setup(&child);
    let args: Vec<String> = cmd
        .get_args()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();
    assert_eq!(cmd.get_program(), child.program.as_os_str());
    assert_eq!(args, cost_setup_args(&model));
    assert_eq!(args[0], "--model=cursor:auto");
    assert_eq!(args[1], "--no-auto-costs");
    assert!(args[2].contains("cursor:auto"));
    assert!(args[2].contains("Do not change repository files"));
}

#[test]
fn failed_spawn_is_one_call_reloads_config_and_warns() {
    let model = cursor();
    let seam = drive(
        &env_for(&model, false),
        vec![open_config(), open_config()],
        CostSetupSpawn::Failed,
    );
    assert_eq!(seam.args, vec![cost_setup_args(&model)]);
    assert_eq!(seam.loads, 2);
    assert!(
        seam.stderr.contains(
            "automatic token-cost setup did not complete; continuing with configured rates"
        )
    );
    assert!(
        seam.stderr
            .contains("token costs remain unset after automatic setup")
    );
}

#[test]
fn completed_spawn_reloads_saved_rates_without_a_warning() {
    let model = cursor();
    let seam = drive(
        &env_for(&model, false),
        vec![open_config(), priced_config()],
        CostSetupSpawn::Completed,
    );
    assert_eq!(seam.args, vec![cost_setup_args(&model)]);
    assert_eq!(seam.loads, 2);
    assert_eq!(seam.stderr, "");
}

#[test]
fn completed_spawn_warns_when_reloaded_rates_stay_unset() {
    let model = cursor();
    let seam = drive(
        &env_for(&model, false),
        vec![open_config(), open_config()],
        CostSetupSpawn::Completed,
    );
    assert_eq!(seam.loads, 2);
    assert_eq!(seam.args.len(), 1);
    assert!(!seam.stderr.contains("did not complete"));
    assert!(
        seam.stderr
            .contains("token costs remain unset after automatic setup")
    );
}

#[test]
fn unlocatable_spawn_warns_once_and_does_not_reload() {
    let model = cursor();
    let seam = drive(
        &env_for(&model, false),
        vec![open_config()],
        CostSetupSpawn::Unlocatable,
    );
    assert_eq!(seam.args.len(), 1);
    assert_eq!(seam.loads, 1);
    assert!(
        seam.stderr
            .contains("could not locate malvin to set up token costs")
    );
    assert!(!seam.stderr.contains("remain unset"));
    assert!(!seam.stderr.contains("did not complete"));
}

#[test]
fn disabled_priced_and_live_models_do_not_spawn() {
    let cursor = cursor();
    let disabled = drive(
        &env_for(&cursor, true),
        vec![open_config()],
        CostSetupSpawn::Completed,
    );
    let priced = drive(
        &env_for(&cursor, false),
        vec![priced_config()],
        CostSetupSpawn::Completed,
    );
    let live = parse_model_id("pi:openrouter/openai/gpt-4o-mini").expect("model");
    let skipped = drive(
        &env_for(&live, false),
        vec![open_config()],
        CostSetupSpawn::Completed,
    );
    for seam in [&disabled, &priced, &skipped] {
        assert_eq!(seam.args, Vec::<Vec<String>>::new());
        assert_eq!(seam.loads, 1);
        assert_eq!(seam.stderr, "");
    }
}
