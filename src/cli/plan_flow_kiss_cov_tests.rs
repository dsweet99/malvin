//! External kiss witnesses for `plan_flow` test helpers.

#[test]
fn kiss_witness_plan_flow_test_helpers() {
    let _ = super::plan_flow_test_helpers::empty_session_dotfile_backups;
    let _ = super::plan_flow_test_helpers::plan_flow_test_prep;
    let _ = super::plan_flow_test_helpers::test_plan_run_prep;
    let _ = super::plan_flow_test_helpers::test_plan_run_prep_for_plan;
    let _ = super::plan_flow_test_helpers::post_1a_content;
    let _ = super::plan_flow_test_helpers::post_1b_content;
    let _ = super::plan_flow_test_helpers::post_2_content;
    let _ = super::plan_flow_test_helpers::plan_shared_opts_for_mock;
    let _ = super::plan_flow_test_helpers::plan_pipeline_mock_handler_body;
    let _ = super::plan_flow_test_helpers::write_plan_pipeline_mock_agent;
    let _ = super::plan_flow_test_helpers::plan_args_for_mock;
    let _ = super::plan_flow_test_helpers::prepare_plan_mock_run;
    let _ = super::plan_flow_test_helpers::prepare_plan_mock_run_with_env;
    let _ = super::plan_flow_test_helpers::install_plan_mock_env;
}

#[test]
fn kiss_witness_plan_flow_mock_tests() {
    let _ = super::plan_flow_mock_tests::prepare_plan_run_truncates_and_loads_prompt_store;
    let _ = super::plan_flow_mock_tests::plan_session_restores_gitignore_after_agent;
    let _ = super::plan_flow_mock_tests::run_plan_gitignore_tamper_prompt;
    let _ = super::plan_flow_mock_tests::run_plan_gitignore_restore_case;
    let _ = super::plan_flow_mock_tests::restore_plan_session_dotfiles;
    let _ = super::plan_flow_mock_tests::write_plan_gitignore_tamper_mock_agent;
    let _ = super::plan_flow_mock_tests::run_plan_acp_mock_agent_prompt_1a_snapshots_artifact;
    let _ = super::plan_flow_mock_tests::run_plan_acp_mock_agent_finalizes_revised_plan;
}
