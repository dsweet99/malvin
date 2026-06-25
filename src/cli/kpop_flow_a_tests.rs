#![allow(unused_imports)]

use super::{
    finish_kpop_prepared, kpop_boot_store_client_prepared, kpop_run_acp_multiturn,
    prepare_kpop_artifacts, run_kpop_short_id_lookup, KpopAcpMultiturnCtx, KpopPrepared,
};
use crate::cli::KpopArgs;
use crate::output::{format_who_tag_prefix, MALVIN_WHO};

fn seed_short_id_lookup_fixture(cwd: &std::path::Path) -> String {
    let run_name = "20260101_000000_abcabcab";
    let bucket = crate::malvin_logs_root(cwd);
    let run_dir = bucket.join(run_name);
    std::fs::create_dir_all(&run_dir).expect("mkdir");
    crate::write_work_dir_manifest(&run_dir, cwd).expect("manifest");
    let exp = run_dir.join("_kpop").join(format!("exp_log_{run_name}.md"));
    std::fs::create_dir_all(exp.parent().unwrap()).expect("mkdir kpop");
    std::fs::write(&exp, "lookup ok\n").expect("write exp");
    let rel = format!("{}/_kpop/exp_log_{run_name}.md", run_dir.display());
    std::fs::write(
        run_dir.join("stdout.log"),
        format!(
            "20260101.000000.000 {}KPOP_LOG: Ma1b2c {rel}\n",
            format_who_tag_prefix(MALVIN_WHO)
        ),
    )
    .expect("stdout");
    rel
}

#[test]
fn prepare_and_finish_kpop_artifacts_skips_nested_gitignore_in_non_git_workspace() {
    crate::test_utils::with_isolated_home(|work| {
        std::fs::create_dir_all(work.join("deep/nested")).expect("mkdir");
        std::fs::write(work.join("deep/.gitignore"), "nested\n").expect("nested gitignore");
        std::fs::create_dir_all(work.join(".malvin")).expect("malvin dir");
        std::fs::write(work.join(".malvin/checks"), "kiss check\n").expect("checks");
        let kpop = KpopArgs {
            max_loops: 1,
            max_hypotheses: 1,
            tenacious: false,
            request: Some("fast startup".into()),
        };
        let early = prepare_kpop_artifacts(&kpop).expect("early artifacts");
        assert_eq!(early.text, "fast startup");
        assert_eq!(
            early.artifacts.work_dir.canonicalize().expect("work_dir"),
            work.canonicalize().expect("work")
        );
        let prepared = finish_kpop_prepared(early).expect("prepared");
        assert_eq!(prepared.text, "fast startup");
        assert!(prepared.context.contains_key("quality_gates"));
        assert!(matches!(
            prepared.session_dotfile_backups.gitignore,
            crate::session_dotfile_backup::GitignoreBackup::Missing
        ));
    });
}

#[test]
fn run_kpop_short_id_lookup_dumps_matching_exp_log() {
    crate::test_utils::with_isolated_home(|cwd| {
        seed_short_id_lookup_fixture(cwd);
        let old = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(cwd).expect("chdir");
        let kpop = KpopArgs {
            max_loops: 1,
            max_hypotheses: 1,
            tenacious: false,
            request: Some("Ma1b2c".into()),
        };
        run_kpop_short_id_lookup(&kpop).expect("lookup dump");
        std::env::set_current_dir(old).expect("restore cwd");
    });
}

#[cfg(unix)]
fn seed_kpop_multiturn_mock_workspace(
    workspace: &std::path::Path,
) -> (std::path::PathBuf, crate::test_utils::SavedEnvVars) {
    use crate::cli::kpop_flow::kpop_flow_run_loop_tests::install_mock_agent_env;

    std::fs::write(workspace.join(".kissconfig"), "k = 1\n").expect("kissconfig");
    let mock = workspace.join("mock-agent");
    let env = install_mock_agent_env(workspace, &mock);
    (mock, env)
}

#[cfg(unix)]
async fn run_kpop_multiturn_mock_once(
    workspace: &std::path::Path,
) -> Result<std::path::PathBuf, String> {
    use crate::cli::kpop_flow::kpop_flow_run_loop::snapshot_kpop_loop_dotfiles_and_exp_log;
    use crate::cli::kpop_flow::kpop_flow_run_loop_tests::test_kpop_args;
    use crate::cli::workflow_kpop_shared::gate_iteration_context;
    use crate::kpop_multiturn_prompts::KpopMultiturnPrompts;
    use crate::kpop_progression::KpopMultiturnState;
    use crate::KpopTurnPrompts;

    let (_mock, _env) = seed_kpop_multiturn_mock_workspace(workspace);
    let (kpop, shared, workflow) = test_kpop_args(1);
    let (store, mut client, prepared) =
        kpop_boot_store_client_prepared(&kpop, &shared, workflow).map_err(|e| e.to_string())?;
    let snap = snapshot_kpop_loop_dotfiles_and_exp_log(&prepared.artifacts, 1, 1)
        .map_err(|e| e.to_string())?;
    let iteration_context = gate_iteration_context(
        &prepared.context,
        &prepared.artifacts,
        &snap.exp_log_path,
        snap.exp_iter,
    );
    let builder = KpopMultiturnPrompts::Turn(KpopTurnPrompts {
        store: &store,
        base: &iteration_context,
        request_text: &prepared.text,
        prepend_rules_once: true,
    });
    let mut state = KpopMultiturnState::new(builder, snap.exp_log_path.clone(), kpop.max_hypotheses)
        .map_err(|e| e.to_string())?;
    kpop_run_acp_multiturn(
        KpopAcpMultiturnCtx {
            client: &mut client,
            prepared: &prepared,
            state: &mut state,
        },
        &snap.backups,
        crate::run_timing::acp_post_run::RunTimingSessionEnd::Finalize,
    )
    .await?;
    Ok(snap.exp_log_path)
}

#[cfg(unix)]
#[test]
fn kpop_run_acp_multiturn_executes_mock_agent() {
    crate::test_utils::enable_test_fast_teardown();
    crate::test_utils::with_isolated_home(|workspace| {
        let exp_log_path = crate::test_utils::block_on_test_async(async {
            run_kpop_multiturn_mock_once(workspace).await
        })
        .expect("multiturn");
        let text = std::fs::read_to_string(exp_log_path).expect("read");
        assert!(text.contains("## Step 1 — KPOP mock"));
    });
}
