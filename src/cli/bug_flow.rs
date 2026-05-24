use crate::orchestrator::format_exp_log_relative;
use crate::output::{MALVIN_WHO, print_stdout_line};
use crate::prompts::PromptStore;
use crate::{malvin_short_id, validate_malvin_short_id};

use super::bug_flow_remediation::{
    finish_bug_remediation, gate_retry_command, run_bug_fix_by_id, BugRunTail, BUG_FOLLOWUP_PLAN,
};
use super::bug_id_lookup::{ensure_exp_log_solved, lookup_bug_id};
use super::SharedOpts;
use super::kpop_flow::{
    KpopAcpMultiturnCtx, KpopPrepared, KpopTurnPrompts, kpop_emit_startup, kpop_run_acp_multiturn,
    prepare_kpop_run,
};
use super::{BugArgs, KpopArgs};
use super::{WorkflowCliOptions, build_agent};

pub(crate) fn kpop_args_from_bug(bug: &BugArgs, request: &str) -> KpopArgs {
    KpopArgs {
        max_hypotheses: bug.max_hypotheses,
        no_learn: bug.no_learn,
        request: Some(request.to_string()),
    }
}

struct BugKpopPhase<'a> {
    kpop: &'a KpopArgs,
    workflow: WorkflowCliOptions,
    store_kpop: &'a PromptStore,
    client: &'a mut crate::acp::AgentClient,
    shared: &'a SharedOpts,
}

async fn run_bug_kpop_multiturn(phase: BugKpopPhase<'_>) -> Result<KpopPrepared, String> {
    use crate::kpop_progression::KpopMultiturnState;
    let prepared = prepare_kpop_run(phase.kpop)?;
    phase.client.prompts_log_run_dir = Some(prepared.artifacts.run_dir.clone());
    super::error_run_log::set_command_error_run_dir(Some(prepared.artifacts.run_dir.clone()));
    kpop_emit_startup(phase.kpop, phase.shared, &prepared.artifacts)?;
    let builder = crate::kpop_multiturn_prompts::KpopMultiturnPrompts::Turn(KpopTurnPrompts {
        store: phase.store_kpop,
        base: &prepared.context,
        request_text: &prepared.text,
        prepend_rules_once: true,
    });
    let mut state = KpopMultiturnState::new(
        builder,
        prepared.exp_log_path.clone(),
        phase.kpop.max_hypotheses,
        0.0,
    )?;
    let acp_result = kpop_run_acp_multiturn(KpopAcpMultiturnCtx {
        client: phase.client,
        prepared: &prepared,
        workflow: phase.workflow,
        state: &mut state,
        store: phase.store_kpop,
    })
    .await;
    crate::acp_post_run::merge_acp_with_workspace_session_restore_and_check_abort(
        acp_result,
        &prepared.artifacts.work_dir,
        &prepared.session_dotfile_backups,
        &prepared.artifacts.artifact_result_md(),
    )?;
    Ok(prepared)
}

fn ensure_kpop_solved(prepared: &KpopPrepared) -> Result<(), String> {
    ensure_exp_log_solved(&prepared.exp_log_path)
}

fn log_bug_id_lines(prepared: &KpopPrepared, id: &str) {
    let rel = format_exp_log_relative(&prepared.artifacts, &prepared.exp_log_path);
    print_stdout_line(MALVIN_WHO, &format!("BUG_ID: {id}"));
    print_stdout_line(MALVIN_WHO, &format!("BUG_LOG: {id} {rel}"));
}

async fn after_kpop_discovery(
    tail: BugRunTail<'_>,
    prepared: KpopPrepared,
    remediate: bool,
    client: &mut crate::acp::AgentClient,
) -> Result<(), String> {
    ensure_kpop_solved(&prepared)?;
    let id = malvin_short_id();
    log_bug_id_lines(&prepared, &id);
    if remediate {
        let artifacts = prepared.into_bug_followup_artifacts(BUG_FOLLOWUP_PLAN)?;
        finish_bug_remediation(tail, artifacts, client, true).await
    } else {
        print_stdout_line(MALVIN_WHO, "DONE");
        Ok(())
    }
}

pub(crate) fn validate_bug_cli(bug: &BugArgs) -> Result<(), String> {
    if bug.fix && bug.bug_id.is_some() {
        return Err("cannot use --fix with a BUG_ID; run `malvin hunt --fix` to discover and fix, or `malvin hunt <id>` to fix an existing discovery".to_string());
    }
    if let Some(ref id) = bug.bug_id {
        validate_malvin_short_id(id).map_err(|_| {
            format!(
                "invalid BUG_ID {id:?}: expected M followed by 5 lowercase letters or digits (example: Ma1b2c)"
            )
        })?;
    }
    Ok(())
}

pub async fn run_bug(
    bug: BugArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    validate_bug_cli(&bug)?;
    let gate_retry_command = gate_retry_command(&bug);
    let tail = BugRunTail {
        bug: &bug,
        shared,
        workflow,
        gate_retry_command,
    };
    let emit_stdout_markdown = shared.acp_stdout_markdown_enabled();
    let mut client = build_agent(shared, workflow, emit_stdout_markdown);
    client.ensure_authenticated().map_err(|e| e.to_string())?;

    let r = if let Some(ref id) = bug.bug_id {
        let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
        let resolved = lookup_bug_id(&cwd, id)?;
        run_bug_fix_by_id(tail, resolved, &mut client).await
    } else {
        let store_kpop = super::prepare_hunt_kpop_prompt_store(workflow)?;
        let request = super::bug_flow_remediation::bug_kpop_request(&store_kpop)?;
        let kpop = kpop_args_from_bug(&bug, &request);
        async {
            let prepared = run_bug_kpop_multiturn(BugKpopPhase {
                kpop: &kpop,
                workflow,
                store_kpop: &store_kpop,
                client: &mut client,
                shared,
            })
            .await?;
            after_kpop_discovery(tail, prepared, bug.fix, &mut client).await
        }
        .await
    };
    if r.is_ok() {
        super::error_run_log::clear_command_error_run_dir();
    }
    r
}

#[cfg(test)]
mod kiss_static_fn_item_refs {
    use super::{after_kpop_discovery, log_bug_id_lines, validate_bug_cli};

    #[test]
    fn kiss_static_fn_item_refs() {
        let _ = log_bug_id_lines;
        let _ = after_kpop_discovery;
        let _ = validate_bug_cli;
    }
}

#[cfg(test)]
mod tests {
    use super::{kpop_args_from_bug, validate_bug_cli, BugArgs};
    use crate::prompts::PromptStore;

    #[test]
    fn kpop_args_from_bug_maps_bug_fields() {
        let bug = BugArgs {
            fix: false,
            max_hypotheses: 7,
            no_learn: true,
            skip_pre_checks: false,
            bug_id: None,
        };
        let store = PromptStore::default_store();
        let expected = super::super::bug_flow_remediation::bug_kpop_request(&store)
            .expect("hunt_request");
        let kpop = kpop_args_from_bug(&bug, &expected);
        assert_eq!(kpop.max_hypotheses, 7);
        assert!(kpop.no_learn);
        assert_eq!(kpop.request.as_deref(), Some(expected.as_str()));
    }

    #[test]
    fn validate_bug_cli_rejects_fix_with_id() {
        let bug = BugArgs {
            fix: true,
            max_hypotheses: 10,
            no_learn: false,
            skip_pre_checks: false,
            bug_id: Some("Ma1b2c".to_string()),
        };
        assert!(validate_bug_cli(&bug).is_err());
    }
}
