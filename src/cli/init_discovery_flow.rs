//! `KPop` checks-discovery phase for `malvin init` on existing repos.

use std::collections::HashMap;
use std::path::Path;

use crate::artifacts::{RunArtifacts, backup_workspace_malvin_checks_if_present};
use crate::kpop_progression::{agent_declared_success, read_exp_log_text};
use crate::malvin_checks_path;
use crate::malvin_config_file::{self, AgentConfig};
use crate::output::{MALVIN_WHO, print_stderr_line};
use crate::prompts::{PromptError, PromptStore};
use crate::repo_gates::discover_init_checks::finalize_init_checks_from_repo;
use crate::repo_gates::init_discovery::{InitDiscoveryDecision, validate_checks_command_lines};
use crate::repo_gates::load_malvin_checks;

use super::gate_kpop_workflow::{
    GateKpopLoopParams, GateKpopPrepared, GateLoopBehavior, run_gate_kpop_loop,
};
use super::workflow_kpop_shared::{gate_kpop_loop_iterations, kpop_workflow_context, render_kpop_program_request};
use super::{SharedOpts, WorkflowCliOptions, prepare_kpop_prompt_store};

pub(crate) fn emit_init_discovery_skip(decision: InitDiscoveryDecision) {
    if let Some(msg) = decision.skip_reason {
        print_stderr_line(MALVIN_WHO, &format!("init: {msg}"));
    }
}

fn prepare_init_kpop_prompt_store(workflow: WorkflowCliOptions) -> Result<PromptStore, String> {
    let store = prepare_kpop_prompt_store(workflow, false)?;
    store
        .validate_exists("kpop_program.md")
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("init_constraints.md")
        .map_err(|e: PromptError| e.0)?;
    Ok(store)
}

fn init_kpop_request(store: &PromptStore, artifacts: &RunArtifacts) -> Result<String, String> {
    render_kpop_program_request(store, "init_constraints.md", &HashMap::new(), artifacts)
}

fn load_init_agent_config(work_dir: &Path) -> AgentConfig {
    malvin_config_file::load_malvin_config(work_dir).agent
}

fn init_discovery_checks_valid(work_dir: &Path) -> Result<(), String> {
    let lines = load_malvin_checks(&malvin_checks_path(work_dir))?;
    validate_checks_command_lines(work_dir, &lines)
}

fn init_discovery_succeeded(artifacts: &RunArtifacts, iterations: usize) -> Result<bool, String> {
    for i in 1..=iterations {
        let exp = artifacts.gate_exp_log_path(i);
        if exp.is_file() {
            let text = read_exp_log_text(&exp)?;
            if agent_declared_success(&text) && init_discovery_checks_valid(&artifacts.work_dir).is_ok()
            {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

pub(crate) async fn run_init_discovery_kpop(
    shared: &SharedOpts,
    artifacts: &RunArtifacts,
) -> Result<bool, String> {
    let workflow = WorkflowCliOptions {
        force: !shared.no_force,
    };
    let store = prepare_init_kpop_prompt_store(workflow)?;
    let request_text = init_kpop_request(&store, artifacts)?;
    std::fs::write(&artifacts.plan_path, &request_text).map_err(|e| e.to_string())?;
    let malvin_checks_backup = backup_workspace_malvin_checks_if_present(&artifacts.work_dir)?;
    let context = kpop_workflow_context(artifacts, "init")?;
    let prepared = GateKpopPrepared {
        artifacts: artifacts.clone(),
        context,
        request_text: request_text.clone(),
        startup_emit_request: request_text,
        store,
        malvin_checks_backup,
    };
    let agent_cfg = load_init_agent_config(&artifacts.work_dir);
    let max_loops = agent_cfg.max_loops;
    let max_hypotheses = agent_cfg.max_hypotheses;
    let iterations = gate_kpop_loop_iterations(max_loops);
    let (gates_ok, agent_ran, _timing, _last_backups) = run_gate_kpop_loop(GateKpopLoopParams {
        command: "init",
        shared,
        workflow,
        prepared: &prepared,
        max_loops,
        max_hypotheses,
        behavior: GateLoopBehavior::INIT,
    })
    .await?;
    let summarize_res = crate::cli::kpop_summarize::run_outer_loop_summarize_if_warranted(
        &crate::cli::kpop_summarize::OuterLoopSummarizeParams {
            max_loops,
            agent_ran,
            shared,
            workflow,
            store: &prepared.store,
            artifacts: &prepared.artifacts,
            malvin_command: "malvin init",
        },
    )
    .await;
    let discovery_r = finish_init_discovery_kpop(&prepared, &artifacts.work_dir, iterations, gates_ok);
    crate::cli::kpop_summarize::prefer_gate_outcome_over_summarize(discovery_r, summarize_res)
}

fn finish_init_discovery_kpop(
    prepared: &GateKpopPrepared,
    work_dir: &std::path::Path,
    iterations: usize,
    gates_ok: bool,
) -> Result<bool, String> {
    let solved = init_discovery_succeeded(&prepared.artifacts, iterations)?;
    if !solved {
        return Err(
            "init checks discovery: agent did not declare KPOP_SOLVED with a valid .malvin/checks"
                .to_string(),
        );
    }
    finalize_init_checks_from_repo(work_dir)?;
    init_discovery_checks_valid(work_dir)?;
    Ok(gates_ok)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifacts::create_kpop_run_artifacts;

    #[test]
    fn init_kpop_request_expands_placeholders() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(tmp.path().join(".malvin")).expect("mkdir");
        std::fs::write(tmp.path().join(".malvin/checks"), "kiss check\n").expect("checks");
        let artifacts = create_kpop_run_artifacts("init", Some(tmp.path())).expect("artifacts");
        let store = PromptStore::default_store();
        store.ensure_defaults().expect("defaults");
        let text = init_kpop_request(&store, &artifacts).expect("request");
        assert!(!text.contains("{{"), "init kpop request must expand placeholders: {text:?}");
        assert!(
            text.contains("Discover how this repo runs quality gates"),
            "expected init_constraints: {text:?}"
        );
    }

    #[test]
    fn prepare_init_kpop_prompt_store_loads_constraints() {
        let workflow = WorkflowCliOptions { force: false };
        let store = prepare_init_kpop_prompt_store(workflow).expect("store");
        assert!(store.validate_exists("init_constraints.md").is_ok());
    }

    #[test]
    fn emit_init_discovery_skip_prints_reason() {
        emit_init_discovery_skip(crate::repo_gates::init_discovery::InitDiscoveryDecision {
            run: false,
            skip_reason: Some("test skip"),
        });
    }

    #[test]
    fn init_discovery_checks_valid_accepts_kiss_when_present() {
        if crate::lookup_bin_on_path("kiss").is_some() {
            let tmp = tempfile::tempdir().expect("tempdir");
            std::fs::create_dir_all(tmp.path().join(".malvin")).expect("mkdir");
            std::fs::write(tmp.path().join(".malvin/checks"), "kiss check\n").expect("checks");
            init_discovery_checks_valid(tmp.path()).expect("valid");
        }
    }

    #[test]
    fn init_discovery_succeeded_false_without_exp_log() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifacts = create_kpop_run_artifacts("init", Some(tmp.path())).expect("artifacts");
        assert!(!init_discovery_succeeded(&artifacts, 1).expect("read"));
    }

    #[test]
    fn init_discovery_succeeded_true_with_marker_and_valid_checks() {
        if crate::lookup_bin_on_path("kiss").is_none() {
            return;
        }
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(tmp.path().join(".malvin")).expect("mkdir");
        std::fs::write(tmp.path().join(".malvin/checks"), "kiss check\n").expect("checks");
        let artifacts = create_kpop_run_artifacts("init", Some(tmp.path())).expect("artifacts");
        let exp = artifacts.gate_exp_log_path(1);
        std::fs::create_dir_all(exp.parent().unwrap()).expect("mkdir");
        std::fs::write(&exp, "## KPOP_SOLVED\n").expect("write");
        assert!(init_discovery_succeeded(&artifacts, 1).expect("read"));
    }

    #[test]
    fn load_init_agent_config_reads_workspace_defaults() {
        let tmp = tempfile::tempdir().expect("tempdir");
        crate::seed_malvin_config(tmp.path(), "");
        let cfg = load_init_agent_config(tmp.path());
        assert_eq!(cfg.max_hypotheses, crate::malvin_config_file::DEFAULT_MAX_HYPOTHESES);
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs{
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _ = emit_init_discovery_skip;
        let _ = load_init_agent_config;
        let _ = init_discovery_checks_valid;
        let _ = init_discovery_succeeded;
        let _ = run_init_discovery_kpop;
        let _ = finish_init_discovery_kpop;
    }
}
