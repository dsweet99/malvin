
use malvin::{canonical_workflow_name, resolve_session_log_path, resolve_workspace_malvin_config_path, RunArtifacts};

#[test]
fn canonical_workflow_name_maps_adaptix_to_inspire() {
    assert_eq!(canonical_workflow_name("adaptix"), "inspire");
    assert_eq!(canonical_workflow_name("inspire"), "inspire");
    assert_eq!(canonical_workflow_name("kpop"), "kpop");
}

#[test]
fn resolve_session_log_path_falls_back_to_adaptix_log() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let run = tmp.path().join("run");
    std::fs::create_dir_all(&run).expect("mkdir");
    let adaptix = run.join("adaptix.log");
    std::fs::write(&adaptix, "legacy transcript\n").expect("write");
    assert_eq!(resolve_session_log_path(&run, "inspire"), adaptix);
}

#[test]
fn resolve_session_log_path_prefers_inspire_log_when_present() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let run = tmp.path().join("run");
    std::fs::create_dir_all(&run).expect("mkdir");
    let inspire = run.join("inspire.log");
    std::fs::write(&inspire, "canonical\n").expect("write");
    std::fs::write(run.join("adaptix.log"), "legacy\n").expect("write");
    assert_eq!(resolve_session_log_path(&run, "inspire"), inspire);
}

#[test]
fn resolve_session_log_path_returns_canonical_when_missing() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let run = tmp.path().join("run");
    std::fs::create_dir_all(&run).expect("mkdir");
    assert_eq!(resolve_session_log_path(&run, "kpop"), run.join("kpop.log"));
}

#[test]
fn resolve_session_log_path_resolves_non_inspire_workflow() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let run = tmp.path().join("run");
    std::fs::create_dir_all(&run).expect("mkdir");
    let kpop = run.join("kpop.log");
    std::fs::write(&kpop, "kpop transcript\n").expect("write");
    assert_eq!(resolve_session_log_path(&run, "kpop"), kpop);
}

#[test]
fn resolve_session_log_path_defaults_to_inspire_log_when_all_missing() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let run = tmp.path().join("run");
    std::fs::create_dir_all(&run).expect("mkdir");
    assert_eq!(
        resolve_session_log_path(&run, "inspire"),
        run.join("inspire.log")
    );
}

#[test]
fn resolve_session_log_path_maps_adaptix_workflow_to_inspire_aliases() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let run = tmp.path().join("run");
    std::fs::create_dir_all(&run).expect("mkdir");
    let adaptix = run.join("adaptix.log");
    std::fs::write(&adaptix, "via adaptix workflow\n").expect("write");
    assert_eq!(resolve_session_log_path(&run, "adaptix"), adaptix);
}

#[test]
fn resolve_workspace_malvin_config_path_prefers_config_toml_over_legacy() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let malvin = tmp.path().join(".malvin");
    std::fs::create_dir_all(&malvin).expect("mkdir");
    let config = malvin.join("config.toml");
    std::fs::write(&config, "theme = \"light\"\n").expect("write");
    std::fs::write(malvin.join("meta.toml"), "theme = \"dark\"\n").expect("write");
    assert_eq!(resolve_workspace_malvin_config_path(tmp.path()), config);
}

#[test]
fn resolve_workspace_malvin_config_path_prefers_meta_toml_when_newer_missing() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let meta = tmp.path().join(".malvin/meta.toml");
    std::fs::create_dir_all(meta.parent().unwrap()).expect("mkdir");
    std::fs::write(&meta, "theme = \"dark\"\n").expect("write");
    assert_eq!(resolve_workspace_malvin_config_path(tmp.path()), meta);
}

#[test]
fn run_artifacts_resolve_session_log_path_matches_helper() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = RunArtifacts {
        run_dir: tmp.path().join("run"),
        plan_path: tmp.path().join("run/plan.md"),
        work_dir: tmp.path().to_path_buf(),
    };
    std::fs::create_dir_all(&artifacts.run_dir).expect("mkdir");
    std::fs::write(artifacts.run_dir.join("adaptix.log"), "x\n").expect("write");
    assert_eq!(
        resolve_session_log_path(&artifacts.run_dir, "inspire"),
        artifacts.run_dir.join("adaptix.log")
    );
}
