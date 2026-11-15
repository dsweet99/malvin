use super::{FactAgentSpec, build_fact_agent_prompt, start_fact_agent_watch_with};
use malvin::artifacts::RunArtifacts;
use malvin::prompts::PromptStore;
use malvin::slow_task_watch::WatchedPromptFlag;
use std::time::Duration;

fn artifacts(dir: &std::path::Path) -> RunArtifacts {
    let run_dir = dir.join("run");
    std::fs::create_dir_all(&run_dir).unwrap();
    RunArtifacts {
        plan_path: run_dir.join("plan.md"),
        run_dir,
        work_dir: dir.to_path_buf(),
    }
}

#[test]
fn prompt_is_header_then_new_fact() {
    let dir = tempfile::tempdir().unwrap();
    let store = PromptStore::default_store();
    let prompt = build_fact_agent_prompt(&store, &artifacts(dir.path()), "m").unwrap();
    let fact = store
        .render_prompt_only("new_fact.md", &std::collections::HashMap::new())
        .unwrap();
    assert!(prompt.contains("# Know thyself"), "{prompt:.80}");
    assert!(prompt.ends_with(fact.trim()));
}

#[tokio::test]
async fn watch_can_be_started_and_dropped() {
    let dir = tempfile::tempdir().unwrap();
    let store = PromptStore::default_store();
    let artifacts = artifacts(dir.path());
    let models = malvin::config::model_list::ModelList::new(vec![
        malvin::config::model_id::parse_model_id(malvin::config::DEFAULT_CLI_MODEL).unwrap(),
    ])
    .unwrap();
    let spec = FactAgentSpec {
        store: &store,
        artifacts: &artifacts,
        models: &models,
    };
    let flag = WatchedPromptFlag::default();
    let _watch = start_fact_agent_watch_with(
        &spec,
        &flag,
        (Duration::from_secs(60), Duration::from_secs(1)),
    );
}
